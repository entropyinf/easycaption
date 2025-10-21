use crate::Res;
use candle_core::Tensor;
use candle_nn::VarBuilder;
use ctc::CTCLoss;
use serde_json::Value;
use std::fs::File;
use std::path::Path;

mod ctc;

#[derive(Debug)]
pub struct DecodeResult {
    pub text: String,
    pub timestamp: (u32, u32),
}

pub struct Decoder {
    ctc: CTCLoss,
    tokens: Value,
}

impl Decoder {
    pub fn new(tokens_file: &dyn AsRef<Path>, vb: VarBuilder) -> Res<Self> {
        let tokens_file = File::open(tokens_file)?;

        let tokens: Value = serde_json::from_reader(tokens_file)?;
        let ctc = CTCLoss::new(25055, 512, true, vb.pp("ctc"))?;
        Ok(Decoder { ctc, tokens })
    }

    pub fn decode(&self, encoder_out: &Tensor) -> Res<Vec<DecodeResult>> {
        let ctc_logits = self.ctc.log_softmax(encoder_out)?;
        println!("ctc_logits: {}", ctc_logits);

        let ids = ctc_logits
            .argmax(2)?
            .flatten(0, 1)?
            .to_vec1::<u32>()?
            .into_iter()
            .filter(|&x| x > 0)
            .collect::<Vec<u32>>();

        let mut results = Vec::<DecodeResult>::new();

        for &x in ids.iter() {
            if let Some(v) = self.tokens.get(x as usize) {
                let text = format!("{}", v.as_str().unwrap_or_default().replace("▁", " "));
                results.push(DecodeResult {
                    text,
                    timestamp: (0, 0),
                });
            }
        }

        Ok(results)
    }
}
