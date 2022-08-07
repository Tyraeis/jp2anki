use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use chrono::prelude::*;

#[derive(
    Serialize, Deserialize, Debug, 
    PartialEq, Eq, PartialOrd, Ord, 
    Hash, Clone, Copy
)]
pub enum Source {
    WaniKani(i32),
    JMDict(i32)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Dictionary {
    pub wanikani_updated_on: Option<DateTime<Utc>>,
    pub jmdict_updated_on: Option<DateTime<Utc>>,
    pub words: HashMap<String, Vec<Source>>,
    pub entries: HashMap<Source, DictionaryEntry>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DictionaryEntry {
    pub forms: Vec<String>,
    pub source: Source,
    pub definitions: Vec<Definition>,
    pub audio: Option<String>,
    pub readings: Vec<String>,
    pub examples: Vec<Example>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Definition {
    pub text: String,
    pub flags: Vec<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Example {
    pub definition: Option<usize>,
    pub en: String,
    pub ja: String
}

impl Dictionary {
    pub fn new() -> Self {
        Dictionary {
            wanikani_updated_on: None,
            jmdict_updated_on: None,
            words: HashMap::new(),
            entries: HashMap::new(),
        }
    }

    pub fn lookup(&self, word: &str) -> impl Iterator<Item=&DictionaryEntry> {
        self.words.get(word).into_iter()
            .flat_map(|indices| {
                indices.iter().copied()
                    .filter_map(|index| self.entries.get(&index))
            })
    }

    pub fn insert(&mut self, entry: DictionaryEntry) {
        for form in entry.forms.iter() {
            let sources = self.words.entry(form.clone())
                .or_default();
            
            let insert_point = sources.partition_point(|s| s < &entry.source);
            sources.insert(insert_point, entry.source);
            sources.dedup();
        }

        self.entries.insert(entry.source, entry);
    }
}

impl DictionaryEntry {

}

impl Definition {
    pub fn new(text: String, flags: Vec<String>) -> Self {
        Definition { text, flags }
    }
}