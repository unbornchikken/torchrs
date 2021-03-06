use nn::{Module, InitModuleStruct, GetFieldStruct, ModDelegate, ModIntf, Parameter};
use autograd::Variable;
use nn::_functions::Conv2dFArgs;
use std::marker::PhantomData;
use nn::functional as F;
use tensor::NumLimits;

#[builder(pattern="owned")]
#[derive(Builder)]
pub struct Conv2dArgs<T: NumLimits> {
    in_features: usize,
    out_features: usize,
    kernel_size: (i32, i32),
    #[builder(default="vec![1, 1]")]
    pub stride: Vec<i32>,
    #[builder(default="vec![0, 0]")]
    pub padding: Vec<i32>,
    #[builder(default="vec![1, 1]")]
    pub dilation: Vec<i32>,
    #[builder(default="1")]
    groups: u32,
    #[builder(default="true")]
    bias: bool,
    #[builder(default="PhantomData")]
    phantom: PhantomData<T>,
}

impl<T: NumLimits> Conv2dArgsBuilder<T> {
    pub fn done(self) -> Conv2d<T> {
        let args = self.build().unwrap();
        Conv2d::new(args)
    }
}

#[derive(ModParse)]
pub struct Conv2d<T: NumLimits> {
    delegate: Module<T>,
    weight: Parameter<T>,
    bias: Option<Parameter<T>>,
    #[ignore]
    args: Conv2dFArgs,
}

impl<T: NumLimits> Conv2d<T> {
    pub fn build(in_features: usize,
                 out_features: usize,
                 kernel_size: (i32, i32))
                 -> Conv2dArgsBuilder<T> {
        Conv2dArgsBuilder::default()
            .in_features(in_features)
            .out_features(out_features)
            .kernel_size(kernel_size)
    }
    fn reset_parameters(mut self) -> Self {
        let mut n = self.args.in_features as i32;
        for k in &self.args.kernel_size {
            n *= *k;
        }
        let stdv: f64 = 1. / (n as f64).sqrt();
        self.weight.v.data().uniform_((-stdv, stdv));
        if let Some(ref mut bias) = self.bias {
            bias.v.data().uniform_((-stdv, stdv));
        }
        self
    }
    pub fn new(args: Conv2dArgs<T>) -> Conv2d<T> {
        let bias = if args.bias {
            Some(Parameter::new((args.out_features)))
        } else {
            None
        };
        let fargs = Conv2dFArgs {
            in_features: args.in_features,
            out_features: args.out_features,
            kernel_size: vec![args.kernel_size.0, args.kernel_size.1],
            stride: args.stride.clone(),
            padding: args.padding.clone(),
            dilation: args.dilation.clone(),
            groups: args.groups,
        };
        Conv2d {
                delegate: Module::new(),
                weight: Parameter::new([args.out_features / args.groups as usize,
                                        args.in_features,
                                        args.kernel_size.0 as usize,
                                        args.kernel_size.1 as usize]),
                bias: bias,
                args: fargs,
            }
            .init_module()
            .reset_parameters()
    }
}
impl_mod_delegate!(Conv2d);

impl<T: NumLimits> ModIntf<T> for Conv2d<T> {
    fn forward(&mut self, input: &mut Variable<T>) -> Variable<T> {
        let bias = if let Some(ref mut biasp) = self.bias {
            Some(&mut biasp.v)
        } else {
            None
        };
        F::conv2d(input, &mut self.weight.v, bias, &mut self.args)
    }
}
