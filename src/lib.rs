use std::mem;
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use lindera::tokenizer::{Tokenizer, Token as LinderaToken};

const IPADICT_POS: usize = 0;
const IPADICT_BASE_FORM: usize = 6;
const IPADICT_READING: usize = 7;

#[derive(Clone, Serialize, Deserialize)]
pub struct Token {
    pub text: String,
    pub pos: String,
    pub base: String,
    pub reading: String
}

impl<'a> From<LinderaToken<'a>> for Token {
    fn from(mut tk: LinderaToken<'a>) -> Token {
        Token {
            text: tk.text.to_owned(),
            pos: mem::take(&mut tk.detail[IPADICT_POS]),
            base: mem::take(&mut tk.detail[IPADICT_BASE_FORM]),
            reading: mem::take(&mut tk.detail[IPADICT_READING])
        }
    }
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn tokenize(text: &str) -> JsValue {
    let tokenizer = Tokenizer::new().unwrap();
    let tokens = tokenizer.tokenize(text).unwrap();

    let tks: Vec<Token> = tokens.into_iter().map(Into::into).collect();
    JsValue::from_serde(&tks).unwrap()
}
