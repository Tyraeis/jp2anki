use std::{collections::HashMap, io::Cursor, cmp::Reverse};
use jp2anki_dict::{DictionaryReader, DictionaryEntry, PartOfSpeech};
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};
use lindera::tokenizer::{Tokenizer, Token};

const IPADICT_POS: usize = 0;
const IPADICT_BASE_FORM: usize = 6;
const IPADICT_READING: usize = 7;

trait TokenExt {
    fn pos(&self) -> &str;
    fn base_form(&self) -> &str;
    fn reading(&self) -> &str;
}

impl<'a> TokenExt for Token<'a> {
    fn pos(&self) -> &str {
        &self.detail[IPADICT_POS]
    }
    fn base_form(&self) -> &str {
        &self.detail[IPADICT_BASE_FORM]
    }
    fn reading(&self) -> &str {
        &self.detail[IPADICT_READING]
    }
}

#[derive(Serialize, Deserialize)]
pub struct AnalyzerResult<'a> {
    word: &'a str,
    pos: PartOfSpeech,
    reading: &'a str,
    count: u32,
    dict_info: Vec<DictionaryEntry>
}

impl<'a> AnalyzerResult<'a> {
    pub fn new(tk: &'a Token<'a>) -> Self {
        AnalyzerResult {
            word: tk.base_form(),
            pos: tk.pos().try_into().unwrap(),
            reading: tk.reading(),
            count: 0,
            dict_info: Default::default()
        }
    }
}

#[wasm_bindgen]
pub fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub struct TextAnalyzer {
    dictionary: DictionaryReader<Cursor<Vec<u8>>>,
}

#[wasm_bindgen]
impl TextAnalyzer {
    #[wasm_bindgen]
    pub fn new(idx_file: Vec<u8>, dat_file: Vec<u8>) -> Self {
        TextAnalyzer {
            dictionary: DictionaryReader::new(
                Cursor::new(idx_file),
                Cursor::new(dat_file)
            ).unwrap()
        }
    }

    #[wasm_bindgen]
    pub fn analyze(&mut self, text: &str) -> JsValue {
        let tokenizer = Tokenizer::new().unwrap();
        let tokens: Vec<Token<'_>> = tokenizer.tokenize(text).unwrap();
    
        let mut words: HashMap<&str, AnalyzerResult<'_>> = HashMap::new();
        for token in &tokens {
            let entry = words.entry(token.base_form())
                .or_insert_with(|| AnalyzerResult::new(token));
            entry.count += 1;
        }
    
        let all_words: Vec<&str> = words.keys().copied().collect();
        for (word, entries) in self.dictionary.lookup(&all_words).unwrap() {
            words.get_mut(word).unwrap().dict_info = entries;
        }

        let mut words: Vec<AnalyzerResult<'_>> = words.into_values().collect();
        words.sort_by_key(|res| Reverse(res.count));

        JsValue::from_serde(&words).unwrap()
    }
}
