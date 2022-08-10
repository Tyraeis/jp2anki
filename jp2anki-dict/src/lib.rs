use std::{collections::{HashMap, BTreeMap, HashSet}, io::{Seek, Write, SeekFrom, Read, Cursor}};
use flate2::{write::DeflateEncoder, Compression, read::DeflateDecoder};
use serde::{Serialize, Deserialize};
use thiserror::Error;

const ENTRIES_PER_CHUNK: usize = 32;
const COMPRESSION_LEVEL: Compression = Compression::best();

#[derive(Debug, Error)]
pub enum DictError {
    #[error("Bincode ser/de error: {0}")]
    Bincode(#[from] bincode::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error)
}

pub type Result<T, E=DictError> = std::result::Result<T, E>;

#[derive(
    Serialize, Deserialize, Debug, 
    PartialEq, Eq, PartialOrd, Ord, 
    Hash, Clone, Copy
)]
pub enum Source {
    WaniKani(i32),
    JMDict(i32)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DictionaryEntry {
    pub forms: Vec<String>,
    pub source: Source,
    pub definitions: Vec<Definition>,
    pub audio: Option<String>,
    pub readings: Vec<String>,
    pub examples: Vec<Example>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Definition {
    pub text: String,
    pub flags: Vec<String>
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Example {
    pub definition: Option<usize>,
    pub en: String,
    pub ja: String
}

impl Definition {
    pub fn new(text: String, flags: Vec<String>) -> Self {
        Definition { text, flags }
    }
}

pub struct DictionaryWriter<W: Write> {
    // BTreeMap ensures keys are kept in order, improving compression efficiency
    index: BTreeMap<String, HashSet<u32>>,
    entry_buffer: Vec<DictionaryEntry>,
    data_position: usize,
    data: W,
}

impl<W: Write> DictionaryWriter<W> {
    pub fn new(data: W) -> Self {
        DictionaryWriter {
            index: Default::default(),
            entry_buffer: Vec::new(),
            data_position: 0,
            data
        }
    }

    pub fn add(&mut self, entry: DictionaryEntry) -> Result<()> {
        for word in &entry.forms {
            self.index.entry(word.clone())
                .or_default()
                .insert(self.data_position as u32);
        }

        self.entry_buffer.push(entry);
        if self.entry_buffer.len() >= ENTRIES_PER_CHUNK {
            self.flush()?;
        }

        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.entry_buffer.len() == 0 {
            return Ok(());
        }

        let entries = bincode::serialize(&self.entry_buffer)?;
        self.entry_buffer.clear();

        let mut enc = DeflateEncoder::new(Vec::new(), COMPRESSION_LEVEL);
        enc.write_all(&entries)?;
        let entries_compressed = enc.finish()?;
        let length_bytes: [u8; 4] = (entries_compressed.len() as u32).to_be_bytes();

        self.data.write_all(&length_bytes)?;
        self.data_position += length_bytes.len();
        self.data.write_all(&entries_compressed)?;
        self.data_position += entries_compressed.len();

        Ok(())
    }

    pub fn finish<W2: Write>(mut self, index_write: W2) -> Result<()> {
        self.flush()?;
        let index_write = DeflateEncoder::new(index_write, COMPRESSION_LEVEL);
        bincode::serialize_into(index_write, &self.index)?;
        Ok(())
    }
}

pub struct DictionaryReader<R: Read + Seek> {
    index: BTreeMap<String, Vec<u32>>,
    data: R
}

impl<R: Read + Seek> DictionaryReader<R> {
    pub fn new(index_read: impl Read, data: R) -> Result<Self> {
        let index_read = DeflateDecoder::new(index_read);
        Ok(DictionaryReader {
            index: bincode::deserialize_from(index_read)?,
            data
        })
    }

    pub fn lookup<'a>(&mut self, words: &[&'a str]) -> Result<HashMap<&'a str, Vec<DictionaryEntry>>> {
        let mut chunks: HashMap<u32, Vec<&'a str>> = HashMap::new();
        for word in words {
            if let Some(entry_chunks) = self.index.get(*word) {
                for chunk_position in entry_chunks {
                    chunks.entry(*chunk_position).or_default().push(word);
                }
            }
        }

        let mut result: HashMap<&'a str, Vec<DictionaryEntry>> = HashMap::new();
        for (chunk_position, words) in &chunks {
            let chunk = self.read_chunk(*chunk_position)?;
            for entry in chunk {
                for word in words {
                    if entry.forms.iter().any(|form| form == word) {
                        result.entry(*word)
                            .or_default()
                            .push(entry.clone());
                    }
                }
            }
        }
        
        Ok(result)
    }

    fn read_chunk(&mut self, pos: u32) -> Result<Vec<DictionaryEntry>> {
        self.data.seek(SeekFrom::Start(pos as u64))?;

        let mut len = [0u8; 4];
        self.data.read_exact(&mut len)?;
        let len = u32::from_be_bytes(len);

        let mut buf = vec![0u8; len as usize];
        self.data.read_exact(&mut buf)?;
        let chunk_read = DeflateDecoder::new(Cursor::new(buf));
        let chunk = bincode::deserialize_from(chunk_read)?;
        Ok(chunk)
    }
}



#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_writer() {
        let mut dat_file = Vec::new();
        let mut idx_file = Vec::new();

        let mut dict_w = DictionaryWriter::new(&mut dat_file);
        dict_w.add(DictionaryEntry {
            forms: vec!["A".into()],
            source: Source::JMDict(1),
            definitions: vec![],
            audio: None,
            readings: vec![],
            examples: vec![]
        }).expect("error in dict_w.add");
        dict_w.finish(&mut idx_file)
            .expect("error in dict_w.finish");

        println!("dat.len={}", dat_file.len());
        println!("idx.len={}", idx_file.len());
        assert!(dat_file.len() > 0);
        assert!(idx_file.len() > 0);

        let len = u32::from_be_bytes(dat_file[0..4].try_into().unwrap());
        assert_eq!(len as usize + 4, dat_file.len());
    }

    #[test]
    fn test_read_write() {
        let mut dat_file = Vec::new();
        let mut idx_file = Vec::new();

        let mut dict_w = DictionaryWriter::new(&mut dat_file);
        dict_w.add(DictionaryEntry {
            forms: vec!["A".into(), "AA".into(), "Q".into()],
            source: Source::JMDict(1),
            definitions: vec![],
            audio: None,
            readings: vec![],
            examples: vec![]
        }).expect("error in dict_w.add#1");
        dict_w.add(DictionaryEntry {
            forms: vec!["X".into(), "YX".into(), "Q".into()],
            source: Source::JMDict(2),
            definitions: vec![],
            audio: None,
            readings: vec![],
            examples: vec![]
        }).expect("error in dict_w.add#2");
        dict_w.finish(&mut idx_file)
            .expect("error in dict_w.finish");

        println!("dat.len={}", dat_file.len());
        println!("idx.len={}", idx_file.len());
        assert!(dat_file.len() > 0);
        assert!(idx_file.len() > 0);

        let mut dict_r = DictionaryReader::new(
            Cursor::new(idx_file),
            Cursor::new(dat_file)
        ).expect("error in DictionaryReader::new");
        let result = dict_r.lookup(&["A", "AA", "X", "Q"])
            .expect("error in dict_r.lookup");

        let result_sources: HashMap<&str, Vec<Source>> = result.into_iter()
            .map(|(word, entries)| {
                let mut sources: Vec<Source> = entries.into_iter().map(|e| e.source).collect();
                sources.sort();
                (word, sources)
            })
            .collect();

        assert_eq!(result_sources["A"], vec![Source::JMDict(1)], "lookup A");
        assert_eq!(result_sources["AA"], vec![Source::JMDict(1)], "lookup AA");
        assert_eq!(result_sources["X"], vec![Source::JMDict(2)], "lookup X");
        assert_eq!(result_sources["Q"], vec![Source::JMDict(1), Source::JMDict(2)], "lookup Q");
    }
}