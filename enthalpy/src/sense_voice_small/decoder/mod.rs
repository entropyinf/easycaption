use crate::Res;
use crate::var_builder::VarBuilder;
use candle_core::Tensor;
use ctc::CTCLoss;
use serde_json::Value;
use std::cmp::max;
use std::fs::File;
use std::path::Path;

mod ctc;

#[derive(Debug)]
pub struct Token {
    pub text: String,
    pub start: u32,
    pub end: u32,
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

    pub fn decode(&self, encoder_out: &Tensor) -> Res<Vec<Token>> {
        let ctc_logits = self.ctc.log_softmax(encoder_out)?;
        let ids = ctc_logits.argmax(2)?;
        let ids = ids.flatten(0, 1)?.to_vec1::<u32>()?;

        let mut results = Vec::<Token>::new();

        let mut start = 0i32;
        let mut active = true;
        for (index, id) in ids.into_iter().enumerate() {
            let index = index as i32;
            if let Some(v) = self.tokens.get(id as usize) {
                let text = format!("{}", v.as_str().unwrap_or_default().replace("▁", " "));

                // build in
                if text.starts_with("<|") {
                    continue;
                }

                if id == 0 {
                    active = true;
                    continue;
                }

                if !active {
                    continue;
                }

                let open = max(start * 60 - 30, 0);
                let close = max(index * 60 - 30, 0);
                start = index;

                results.push(Token {
                    text,
                    start: open as u32,
                    end: close as u32,
                });
                active = false;
            }
        }

        Ok(results)
    }
}
