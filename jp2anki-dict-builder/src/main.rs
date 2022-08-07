#![feature(iter_intersperse)]

mod wanikani;
mod jmdict;

use std::{path::Path, fs::File};
use jp2anki_dict::Dictionary;
use chrono::prelude::*;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "jp2anki dictionary builder")]
struct Args {
    #[clap(short, long, value_parser)]
    token: Option<String>,
    #[clap(short, long, value_parser)]
    jmdict_path: Option<String>,
    #[clap(short, long, value_parser, default_value_t = String::from("dictionary.bin"))]
    path: String,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.path);
    let mut dict = path.exists()
        .then_some(())
        .and_then(|_| File::open(path).ok())
        .and_then(|f| bincode::deserialize_from(f).ok())
        .unwrap_or_else(|| {
            println!("Couldn't open dictionary; creating new dictionary...");
            Dictionary::new()
        });
        
    if let Some(token) = &args.token {
        println!("Updating WaniKani entries...");
        let last_updated = dict.wanikani_updated_on.clone();
        wanikani::update_wanikani(&mut dict, last_updated, token).unwrap();
        dict.wanikani_updated_on = Some(Utc::now());
    }

    if let Some(ref jmdict_path) = args.jmdict_path {
        let jmdict_path = Path::new(jmdict_path);
        if jmdict_path.exists() {
            println!("Updating JMDict entries...");
            jmdict::update_jmdict(&mut dict, jmdict_path).unwrap();
            dict.jmdict_updated_on = Some(Utc::now());
        }
    }

    println!("Saving dictionary...");
    let f = File::create(path).unwrap();
    bincode::serialize_into(f, &dict).unwrap();
}
