use std::io::Write;

use jp2anki_dict::{DictionaryEntry, Definition, Example, Source, DictionaryWriter};
use anyhow::{Result, anyhow};
use serde::{Deserialize, de::DeserializeOwned};
use reqwest::{blocking::{Client, RequestBuilder}, StatusCode};

const WK_SUBJECTS_ENDPOINT: &str = "https://api.wanikani.com/v2/subjects";
const WK_REVISION: &str = "20170710";

#[derive(Deserialize)]
pub struct WkSubjects {
    pub pages: Option<WkPagination>,
    pub total_count: Option<i32>,
    pub data_updated_at: Option<String>,
    pub data: Vec<WkSubject>
}

#[derive(Deserialize)]
pub struct WkPagination {
    pub per_page: i32,
    pub next_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(tag = "object")]
#[serde(rename_all = "lowercase")]
pub enum WkSubject {
    Radical,
    Kanji,
    Vocabulary {
        id: i32,
        data: WkVocab
    },
}

#[derive(Deserialize)]
pub struct WkVocab {
    pub characters: String,
    pub meanings: Vec<WkMeaning>,
    pub context_sentences: Vec<WkContextSentence>,
    pub parts_of_speech: Vec<String>,
    pub pronunciation_audios: Vec<WkPronunciationAudio>,
    pub readings: Vec<WkReading>
}

#[derive(Deserialize)]
pub struct WkMeaning {
    pub meaning: String,
}

#[derive(Deserialize)]
pub struct WkContextSentence {
    pub en: String,
    pub ja: String
}

#[derive(Deserialize)]
pub struct WkPronunciationAudio {
    pub url: String,
    pub content_type: String,
    pub metadata: WkPronunciationAudioMetadata
}

#[derive(Deserialize)]
pub struct WkPronunciationAudioMetadata {
    pub pronunciation: String
}

#[derive(Deserialize)]
pub struct WkReading {
    pub reading: String
}

impl WkSubject {
    pub fn into_dictionary_entry(self) -> Result<Option<DictionaryEntry>> {
        match self {
            WkSubject::Vocabulary { id, data } => {
                let WkVocab {
                    characters, meanings,
                    readings, pronunciation_audios,
                    context_sentences, ..
                } = data;

                let definition = meanings.iter()
                    .map(|meaning| meaning.meaning.as_str())
                    .intersperse(", ")
                    .collect::<String>();

                let audio = pronunciation_audios.into_iter()
                    .map(|audio| audio.url)
                    .collect();

                let readings = readings.into_iter()
                    .map(|reading| reading.reading)
                    .collect();

                let examples = context_sentences.into_iter()
                    .map(|sentence| {
                        Example {
                            for_definition: None,
                            en: sentence.en,
                            ja: sentence.ja
                        }
                    })
                    .collect();

                let definition = Definition::new(definition, data.parts_of_speech, vec!["wk".into()])?;

                Ok(Some(DictionaryEntry {
                    forms: vec![characters],
                    source: Source::WaniKani(id),
                    definitions: vec![definition],
                    audio,
                    readings,
                    examples,
                }))
            },
            _ => Ok(None)
        }
    }
}

pub struct WkClient {
    client: Client,
    token: String
}

impl WkClient {
    pub fn new(token: &str) -> Self {
        WkClient { client: Client::new(), token: token.to_owned() }
    }

    pub fn get(&self, url: &str) -> WkRequest {
        let req = self.client.get(url)
            .header("Wanikani-Revision", WK_REVISION)
            .bearer_auth(&self.token);
        WkRequest(req)
    }
}

pub struct WkRequest(RequestBuilder);

impl WkRequest {
    pub fn send<T: DeserializeOwned>(self) -> Result<T> {
        let resp = self.0.send()?;
        if resp.status() == StatusCode::OK {
            Ok(resp.json()?)
        } else {
            Err(anyhow!("Error {}: {}", resp.status(), resp.text()?))
        }
    }
}

pub fn update_wanikani<W: Write>(dict: &mut DictionaryWriter<W>, token: &str) -> Result<()> {
    let client = WkClient::new(token);
    let request = client.get(WK_SUBJECTS_ENDPOINT);

    let mut subjects: WkSubjects = request.send()?;
    loop {
        println!("Found {} updated WaniKani entries", subjects.data.len());
        for subject in subjects.data {
            match subject.into_dictionary_entry() {
                Ok(Some(entry)) => dict.add(entry)?,
                Ok(None) => (),
                Err(e) => eprintln!("{}", e)
            }
        }

        if let Some(next_url) = subjects.pages.and_then(|p| p.next_url) {
            subjects = client.get(&next_url).send()?
        } else {
            break
        }
    }

    Ok(())
}
