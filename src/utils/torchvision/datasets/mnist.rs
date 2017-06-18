#![allow(deprecated)]
use std::ops::Index;
use std::path::PathBuf;
use std::{io, fs, slice, cell};
use curl::easy::Easy;
use memmap::{Mmap, Protection};
use flate2::{Flush, Decompress};
use std::io::Write;
use utils::data::{Dataset, DatasetIntf};
use std::rc::Rc;
use tensor::{Tensor, TensorKind};
use torch;

type Sample = (TensorKind, i64);

static URLS: [&str; 4] = ["http://yann.lecun.com/exdb/mnist/train-images-idx3-ubyte.gz",
                          "http://yann.lecun.com/exdb/mnist/train-labels-idx1-ubyte.gz",
                          "http://yann.lecun.com/exdb/mnist/t10k-images-idx3-ubyte.gz",
                          "http://yann.lecun.com/exdb/mnist/t10k-labels-idx1-ubyte.gz"];
static RAW_FOLDER: &str = "raw";
static PROCESSED_FOLDER: &str = "processed";
static TRAINING_FILE: &str = "training.pt";
static TEST_FILE: &str = "test.pt";

fn create_dir_f(arg: PathBuf) -> io::Result<()> {
    let result = fs::create_dir(arg);
    if let Some(err) = result.err() {
        match err.kind() {
            io::ErrorKind::AlreadyExists => Ok(()),
            _ => Err(err),
        }
    } else {
        Ok(())
    }
}

fn check_exists(arg: &PathBuf) -> io::Result<bool> {
    let train = fs::metadata(arg.join(TRAINING_FILE))?;
    let test = fs::metadata(arg.join(TEST_FILE))?;

    return Ok(train.is_file() && test.is_file());
}

fn download(root: &String) -> io::Result<()> {
    let raw_path = PathBuf::from(root).join(RAW_FOLDER);
    let processed_path = PathBuf::from(root).join(PROCESSED_FOLDER);
    create_dir_f(raw_path.clone())?;
    create_dir_f(processed_path.clone())?;

    for url in URLS.iter() {
        println!("downloading {}", url);
        let paths: Vec<_> = url.split("/").collect();
        let fname: Vec<_> = paths[5].split("/").collect();
        let file_path = raw_path.join(fname[0]);
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;
        let mut data = Vec::new();
        let mut handle = Easy::new();
        handle.url(url).unwrap();
        {
            let mut transfer = handle.transfer();
            transfer
                .write_function(|new_data| {
                                    data.extend_from_slice(new_data);
                                    Ok(new_data.len())
                                })
                .unwrap();
            transfer.perform().unwrap();
        }
        let mut output = Vec::with_capacity(data.len());
        // do we have a header?
        Decompress::new(false)
            .decompress_vec(data.as_slice(), &mut output, Flush::Finish)?;
        file.write_all(output.as_slice())?;
    }
    println!("Proceeding");

    let training_set = vec![read_image_file(raw_path.join("train-images-idx3-ubyte"))?,
                            read_label_file(raw_path.join("train-labels-idx1-ubyte"))?];
    let test_set = vec![read_image_file(raw_path.join("t10k-images-idx3-ubyte"))?,
                        read_label_file(raw_path.join("t10k-labels-idx1-ubyte"))?];

    torch::save(processed_path.join(TRAINING_FILE), &training_set)?;
    torch::save(processed_path.join(TEST_FILE), &test_set)?;

    Ok(())
}

fn read_image_file(path: PathBuf) -> io::Result<TensorKind> {
    let fp = Mmap::open_path(path, Protection::Read)?;
    let (length, nrows, ncols);
    {
        let data = unsafe { slice::from_raw_parts(fp.ptr() as *const u32, fp.len()) };
        assert_eq!(data[0], 2051);
        length = data[1] as usize;
        nrows = data[2] as usize;
        ncols = data[3] as usize;
    }
    let data = unsafe { fp.as_slice() };
    let mut images: Vec<Vec<u8>> = Vec::with_capacity(length);
    let mut idx = 16;
    for _ in 0..length {
        let mut img = Vec::with_capacity(nrows * ncols);
        for _ in 0..nrows * ncols {
            img.push(data[idx]);
            idx += 1;
        }
        images.push(img)
    }
    Ok(torch::byte_tensor(images).view(&[-1, 28, 28]))
}

fn read_label_file(path: PathBuf) -> io::Result<TensorKind> {
    let fp = Mmap::open_path(path, Protection::Read)?;
    let length;
    {
        let data = unsafe { slice::from_raw_parts(fp.ptr() as *const u32, fp.len()) };
        assert_eq!(data[0], 20549);
        length = data[1] as usize;
    }
    let data = unsafe { fp.as_slice() };
    let mut labels: Vec<i64> = Vec::with_capacity(length);
    for i in 0..length {
        labels.push(data[8 + i] as i64)
    }
    Ok(torch::long_tensor(labels))
}

pub struct MNIST {
    pub root: String,
    pub train: bool,
    pub data: Tensor<u8>,
    pub labels: Tensor<i64>,
    pub transformed: cell::RefCell<Vec<(TensorKind, i64)>>,
    pub transform: Option<Xfrm>,
}

#[builder(pattern="owned")]
#[derive(Builder)]
pub struct MNISTArgs {
    root: String,
    #[builder(default="true")]
    train: bool,
    #[builder(default="false")]
    download: bool,
}

type Xfrm = Box<fn(&TensorKind) -> TensorKind>;
impl MNISTArgsBuilder {
    pub fn done(self, xfrm: Option<Xfrm>) -> Dataset<Sample> {
        let args = self.build().unwrap();
        Dataset::new(Rc::new(MNIST::new(&args, xfrm)))
    }
}

impl MNIST {
    pub fn build(root: &str) -> MNISTArgsBuilder {
        MNISTArgsBuilder::default().root(root.into())
    }
    pub fn new(args: &MNISTArgs, xfrm: Option<Xfrm>) -> Self {
        if args.download {
            download(&args.root).expect("download failed");
        }
        let processed_path = PathBuf::from(&args.root).join(PROCESSED_FOLDER);
        if !check_exists(&processed_path).expect("Dataset not found, try downloading") {
            panic!("Dataset not found, use download=true to download it");
        }
        let mut v: Vec<TensorKind> = if args.train {
            torch::load(processed_path.join(TRAINING_FILE)).expect("torch load failed")
        } else {
            torch::load(processed_path.join(TEST_FILE)).expect("torch load failed")
        };
        let (data, labels) = (v.remove(0), v.remove(0));
        let count = data.size()[0];
        let mut mutable: Vec<(TensorKind, i64)> = Vec::with_capacity(count);
        for i in 0..count {
            mutable.push((data.s(i).copy().into(), i as i64))
        }

        MNIST {
            root: args.root.clone(),
            train: args.train,
            data: data.into(),
            labels: labels.into(),
            transformed: cell::RefCell::new(mutable),
            transform: xfrm,
        }
    }
}

impl Index<usize> for MNIST {
    type Output = Sample;
    fn index(&self, idx: usize) -> &Self::Output {
        let img = self.data.s(idx);
        let xfrmp = self.transformed.as_ptr();
        let mut xfrm = unsafe { &mut *xfrmp };

        xfrm[idx] = (img.copy().into(), self.labels[idx] as i64);
        &xfrm[idx]
    }
}

impl DatasetIntf<Sample> for MNIST {
    fn len(&self) -> usize {
        if self.train { 60000 } else { 10000 }
    }
    fn iter(&self) -> Box<Iterator<Item = Sample>> {
        unimplemented!();
    }
}