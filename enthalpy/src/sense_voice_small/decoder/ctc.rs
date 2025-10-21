use candle_core::{Result, Tensor};
use candle_nn::{Linear, Module, VarBuilder};
use crate::Res;

#[derive(Debug, Clone)]
pub struct CTCLoss {
    ctc_lo: Option<Linear>,
}

impl CTCLoss {
    /// Create a new CTC instance
    ///
    /// # Arguments
    /// * `odim` - Output dimension
    /// * `encoder_output_size` - Encoder output size
    /// * `dropout_rate` - Dropout rate (0.0 ~ 1.0)
    /// * `ctc_type` - CTC type ("builtin", etc.)
    /// * `reduce` - Whether to reduce CTC loss to scalar
    /// * `ignore_nan_grad` - Whether to ignore NaN gradients
    /// * `extra_linear` - Whether to use an extra linear layer
    pub fn new(
        odim: usize,
        encoder_output_size: usize,
        extra_linear: bool,
        vb: VarBuilder,
    ) -> Res<Self> {
        let ctc_lo = if extra_linear {
            Some(candle_nn::linear(
                encoder_output_size,
                odim,
                vb.pp("ctc_lo"),
            )?)
        } else {
            None
        };

        Ok(Self {
            ctc_lo,
        })
    }


    /// Apply log_softmax to frame activations
    ///
    /// # Arguments
    /// * `hs_pad` - 3D tensor (B, Tmax, eprojs)
    /// # Returns
    /// * 3D tensor with log softmax applied (B, Tmax, odim)
    pub fn log_softmax(&self, hs_pad: &Tensor) -> Result<Tensor> {
        if let Some(ctc_lo) = &self.ctc_lo {
            candle_nn::ops::log_softmax(&ctc_lo.forward(hs_pad)?, 2)
        } else {
            candle_nn::ops::log_softmax(hs_pad, 2)
        }
    }

    pub fn softmax(&self, hs_pad: &Tensor) -> Result<Tensor> {
        if let Some(ctc_lo) = &self.ctc_lo {
            candle_nn::ops::softmax(&ctc_lo.forward(hs_pad)?, 2)
        } else {
            candle_nn::ops::softmax(hs_pad, 2)
        }
    }
}