// Maintain the Torch names
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]

use tensor::TensorKind;
use nn::backends::backend::*;
use rutorch::*;


pub struct THNN_FloatBackend {
    state: *mut ::std::os::raw::c_void,
}

impl BackendIntfActivation for THNN_FloatBackend {
    fn Threshold_updateOutput(&self,
                              input: &mut TensorKind,
                              output: &mut TensorKind,
                              threshold_: f32,
                              val_: f32,
                              inplace: bool) {

    }
    fn Threshold_updateGradInput(&self,
                                 input: &mut TensorKind,
                                 grad_output: &mut TensorKind,
                                 grad_input: &mut TensorKind,
                                 threshold_: f32,
                                 val_: f32,
                                 inplace: bool) {

    }
}

impl BackendIntfLoss for THNN_FloatBackend {
    fn ClassNLLCriterion_updateOutput(&self,
                                      input: &TensorKind,
                                      target: &TensorKind,
                                      output: &mut TensorKind,
                                      size_average: bool,
                                      weights: Option<&TensorKind>,
                                      total_weight: &TensorKind) {

    }
    fn ClassNLLCriterion_updateGradInput(&self,
                                         input: &TensorKind,
                                         target: &TensorKind,
                                         grad_input: &mut TensorKind,
                                         size_average: bool,
                                         weights: Option<&TensorKind>,
                                         total_weight: &TensorKind) {
    }
    fn LogSoftMax_updateOutput(&self, input: &TensorKind, output: &mut TensorKind) {}
    fn LogSoftMax_updateGradInput(&self,
                                  input: &TensorKind,
                                  grad_output: &TensorKind,
                                  grad_input: &mut TensorKind,
                                  output: &TensorKind) {
    }
}

impl BackendIntfPooling for THNN_FloatBackend {
    fn SpatialDilatedMaxPooling_updateOutput(&self,
                                             input: &TensorKind,
                                             output: &mut TensorKind,
                                             indices: &mut TensorKind,
                                             kernel_size: (i32, i32),
                                             stride: (i32, i32),
                                             padding: (i32, i32),
                                             dilation: (i32, i32),
                                             ceil_mode: bool) {
        unsafe {
            THNN_FloatSpatialDilatedMaxPooling_updateOutput(self.state,
                                                            input.in_thft(),
                                                            output.in_thft(),
                                                            indices.in_thlt(),
                                                            kernel_size.0,
                                                            kernel_size.1,
                                                            stride.0,
                                                            stride.1,
                                                            padding.0,
                                                            padding.1,
                                                            dilation.0,
                                                            dilation.1,
                                                            ceil_mode);
        }

    }

    fn SpatialDilatedMaxPooling_updateGradInput(&self,
                                                input: &TensorKind,
                                                grad_output: &TensorKind,
                                                grad_input: &mut TensorKind,
                                                indices: &TensorKind,
                                                kernel_size: (i32, i32),
                                                stride: (i32, i32),
                                                padding: (i32, i32),
                                                dilation: (i32, i32),
                                                ceil_mode: bool) {

    }
}

impl BackendIntf for THNN_FloatBackend {}