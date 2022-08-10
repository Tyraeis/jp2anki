#![feature(iter_intersperse)]

mod wanikani;
mod jmdict;

use std::{path::Path, fs::File, io::BufWriter};
use jp2anki_dict::DictionaryWriter;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(name = "jp2anki dictionary builder")]
struct Args {
    #[clap(short, long, value_parser)]
    token: Option<String>,
    #[clap(short, long, value_parser)]
    jmdict_path: Option<String>,
    #[clap(short, long, value_parser, default_value_t = String::from("dictionary"))]
    dict_name: String,
}

fn main() {
    let args = Args::parse();

    let path = Path::new(&args.dict_name);
    let dat_fp = File::create(&path.with_extension("dat")).unwrap();
    let mut dict = DictionaryWriter::new(BufWriter::new(dat_fp));
        
    if let Some(token) = &args.token {
        println!("Updating WaniKani entries...");
        wanikani::update_wanikani(&mut dict, token).unwrap();
    }

    if let Some(ref jmdict_path) = args.jmdict_path {
        let jmdict_path = Path::new(jmdict_path);
        if jmdict_path.exists() {
            println!("Updating JMDict entries...");
            jmdict::update_jmdict(&mut dict, jmdict_path).unwrap();
        }
    }

    println!("Saving dictionary...");
    let idx_fp = File::create(path.with_extension("idx")).unwrap();
    dict.finish(idx_fp).unwrap();
}
