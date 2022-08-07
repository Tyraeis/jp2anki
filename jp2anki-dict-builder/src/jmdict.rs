use std::{fs::File, io::{Read, BufRead}, path::Path};

use jp2anki_dict::{DictionaryEntry, Definition, Example, Source, Dictionary};
use quick_xml::{events::{Event, BytesStart, attributes::Attribute, BytesEnd}, Reader};
use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JMDictParseError {
    #[error("Mismatched start and end tags: start tag {start:?} does not match end tag {end:?}")]
    MismatchedTags {
        start: String,
        end: String
    },
    #[error("Unexpected start tag {tag_name:?} (in tag {in_tag:?})")]
    UnexpectedStart {
        tag_name: String,
        in_tag: String,
    }
}

impl JMDictParseError {
    pub fn mismatched_tags(start: &BytesStart, end: &BytesEnd) -> Self {
        JMDictParseError::MismatchedTags {
            start: String::from_utf8_lossy(start.name()).into_owned(),
            end: String::from_utf8_lossy(end.name()).into_owned(),
        }
    }

    pub fn unexpected_start(tag: &BytesStart, parent: &BytesStart) -> Self {
        JMDictParseError::UnexpectedStart {
            tag_name: String::from_utf8_lossy(tag.name()).into_owned(),
            in_tag: String::from_utf8_lossy(parent.name()).into_owned(),
        }
    }
}

trait XmlObject: Sized {
    fn parse_xml<T: BufRead>(&mut self, rdr: &mut Reader<T>, tag: &BytesStart) -> Result<()>;
    fn from_attr(&mut self, _attr: &Attribute) -> Result<()> { Ok(()) }
}

impl XmlObject for String {
    fn parse_xml<T: BufRead>(&mut self, rdr: &mut Reader<T>, tag: &BytesStart) -> Result<()> {
        let mut buf = Vec::new();
        loop {
            match rdr.read_event(&mut buf)? {
                Event::Text(ref e) => *self = std::str::from_utf8(e)?.to_string(),
                Event::Start(ref e) =>
                    return Err(JMDictParseError::unexpected_start(e, tag).into()),
                Event::End(ref e) if e.name() == tag.name() =>
                    return Ok(()),
                Event::End(ref e) =>
                    return Err(JMDictParseError::mismatched_tags(tag, e).into()),
                _ => ()
            }
        }
    }

    fn from_attr(&mut self, attr: &Attribute) -> Result<()> {
        *self = std::str::from_utf8(&attr.value)?.to_string();
        Ok(())
    }
}

impl<T> XmlObject for Vec<T> where T: XmlObject + Default {
    fn parse_xml<R: BufRead>(&mut self, rdr: &mut Reader<R>, tag: &BytesStart) -> Result<()> {
        let mut value = T::default();
        value.parse_xml(rdr, tag)?;
        self.push(value);
        Ok(())
    }
}

impl<T> XmlObject for Option<T> where T: XmlObject + Default {
    fn parse_xml<R: BufRead>(&mut self, rdr: &mut Reader<R>, tag: &BytesStart) -> Result<()> {
        let mut value = T::default();
        value.parse_xml(rdr, tag)?;
        *self = Some(value);
        Ok(())
    }

    fn from_attr(&mut self, attr: &Attribute) -> Result<()> {
        let mut value = T::default();
        value.from_attr(attr)?;
        *self = Some(value);
        Ok(())
    }
}

impl XmlObject for () {
    fn parse_xml<T: BufRead>(&mut self, rdr: &mut Reader<T>, tag: &BytesStart) -> Result<()> {
        let mut buf = Vec::new();
        loop {
            match rdr.read_event(&mut buf)? {
                Event::Start(ref e) => ().parse_xml(rdr, e)?,
                Event::End(ref e) if e.name() == tag.name() =>
                    return Ok(()),
                Event::End(ref e) =>
                    return Err(JMDictParseError::mismatched_tags(tag, e).into()),
                _ => ()
            }
        }
    }
}

macro_rules! xml_struct {
    ($(
        $vis:vis struct $name:ident {
            $(( self ) $self_field:ident : String,)?
            $(( $tag_name:literal ) $field:ident : $ty:ty),* $(,)?
        }
    )*) => {$(
        #[derive(Debug, Default)]
        $vis struct $name {
            $($self_field: String,)?
            $($field: $ty),*
        }

        impl XmlObject for $name {
            fn parse_xml<T: BufRead>(&mut self, rdr: &mut Reader<T>, tag: &BytesStart) -> Result<()> {
                for attr in tag.attributes() {
                    let attr = attr?;
                    match attr.key {
                        $($tag_name => <$ty as XmlObject>::from_attr(&mut self.$field, &attr)?,)*
                        _ => ()
                    }
                }

                let mut buf = Vec::new();
                loop {
                    match rdr.read_event(&mut buf)? {
                        $(Event::Text(ref e) =>
                            self.$self_field = String::from_utf8(e.to_vec())?,
                        )?
                        $(Event::Start(ref e) if e.name() == $tag_name =>
                            <$ty as XmlObject>::parse_xml(&mut self.$field, rdr, e)?,
                        )*
                        Event::Start(ref e) => ().parse_xml(rdr, e)?,
                        Event::End(ref e) if e.name() == tag.name() =>
                            return Ok(()),
                        Event::End(ref e) =>
                            return Err(JMDictParseError::mismatched_tags(tag, e).into()),
                        _ => ()
                    }
                }
            }
        }
    )*}
}

xml_struct! {
    struct JMDict {
        (b"entry")entries: Vec<JMDEntry>
    }

    struct JMDEntry {
        (b"ent_seq")ent_seq: String,
        (b"k_ele")k_ele: Vec<JMDKanji>,
        (b"r_ele")r_ele: Vec<JMDReading>,
        (b"sense")sense: Vec<JMDSense>
    }
    
    struct JMDKanji {
        (b"keb")keb: String,
        (b"ke_inf")ke_inf: Vec<String>,
        (b"ke_pri")ke_pri: Vec<String>
    }
    
    struct JMDReading {
        (b"reb")reb: String,
        (b"re_nokanji")re_nokanji: Option<String>,
        (b"re_restr")re_restr: Vec<String>,
        (b"re_inf")re_inf: Vec<String>,
        (b"re_pri")re_pri: Vec<String>
    }
    
    struct JMDSense {
        (b"stagk")stagk: Vec<String>,
        (b"stagr")stagr: Vec<String>,
        (b"pos")pos: Vec<String>,
        (b"xref")xref: Vec<String>,
        (b"ant")ant: Vec<String>,
        (b"field")field: Vec<String>,
        (b"misc")misc: Vec<String>,
        (b"s_inf")s_inf: Vec<String>,
        (b"lsource")lsource: Vec<JMDLSource>,
        (b"dial")dial: Vec<String>,
        (b"gloss")gloss: Vec<JMDGloss>,
        (b"example")examples: Vec<JMDExample>
    }

    struct JMDLSource {
        (self)lsource: String,
        (b"lang")lang: String,
        (b"ls_type")ls_type: Option<String>,
        (b"ls_wasei")ls_wasei: Option<String>,
    }

    struct JMDGloss {
        (self)text: String,
        (b"xml:lang")lang: Option<String>,
        (b"g_gend")g_gend: Option<String>
    }

    struct JMDExample {
        (b"ex_srce")ex_srce: String,
        (b"ex_text")ex_text: String,
        (b"ex_sent")ex_sent: Vec<JMDSentence>
    }

    struct JMDSentence {
        (self)text: String,
        (b"xml:lang")lang: String
    }
}

impl JMDEntry {
    fn into_dictionary_entry(self) -> DictionaryEntry {
        let forms = self.k_ele.into_iter()
            .map(|kanji| kanji.keb)
            .collect();

        let definitions = self.sense.iter()
            .map(|sense| {
                let definition = sense.gloss.iter()
                    .map(|gloss| gloss.text.as_str())
                    .intersperse(", ")
                    .collect();
                
                let mut flags = sense.misc.clone();
                flags.extend(sense.dial.iter().cloned());
                flags.extend(sense.field.iter().cloned());

                Definition::new(definition, flags)
            })
            .collect();

        let readings = self.r_ele.into_iter()
            .map(|reading| reading.reb)
            .collect();
        
        let examples = self.sense.into_iter()
            .enumerate()
            .flat_map(|(i, sense)| {
                sense.examples.into_iter()
                    .filter_map(move |example| {
                        let en = example.ex_sent.iter()
                            .find(|sent| sent.lang == "eng");
                        
                        let ja = example.ex_sent.iter()
                            .find(|sent| sent.lang == "jpn");

                        if let (Some(en), Some(ja)) = (en, ja) {
                            Some(Example {
                                definition: Some(i),
                                en: en.text.clone(),
                                ja: ja.text.clone()
                            })
                        } else {
                            None
                        }
                    })
            })
            .collect();

        DictionaryEntry {
            forms,
            source: Source::JMDict(self.ent_seq.trim().parse().unwrap_or(-1)),
            definitions,
            audio: None,
            readings,
            examples 
        }
    }
}

pub fn update_jmdict<P: AsRef<Path>>(dict: &mut Dictionary, path: P) -> Result<()> {
    let mut fp = File::open(path)?;
    let mut buf = Vec::new();
    fp.read_to_end(&mut buf)?;

    let mut pos = 2;
    while pos < buf.len() {
        if &buf[pos-2..pos] == b"]>" { break }
        pos += 1;
    }

    let mut reader = Reader::from_bytes(&buf[pos..]);
    let mut buf2 = Vec::new();

    println!("Reading JMDict file...");
    let mut jmdict = JMDict::default();
    loop {
        match reader.read_event(&mut buf2)? {
            Event::Start(ref e) => {
                jmdict.parse_xml(&mut reader, e)?;
                break;
            }
            _ => ()
        }
    }
    for entry in jmdict.entries {
        dict.insert(entry.into_dictionary_entry())
    }

    Ok(())
}
