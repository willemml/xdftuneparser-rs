#![feature(if_let_guard)]
#![feature(decl_macro)]

use std::io::BufReader;
use std::{fs::File, io::Read};

use xml::reader::EventReader;
use xml::ParserConfig;

pub mod data_types;
pub mod error;
pub mod parser;

use data_types::*;

// table has 3 axis, x and y are column and row "headers", z is actual value to be written, z can be a function of x and y?

// data written seems to always be one dimensional, xdf contains extras for a fake 3d table (2d where one dimension has two values per point) to make it more obvious what you are editing

fn main() -> std::io::Result<()> {
    let mut binfile = File::open("file.bin")?;
    let mut bin = Vec::new();
    binfile.read_to_end(&mut bin)?;

    let file = File::open("file.xml")?;
    let file: BufReader<File> = BufReader::new(file); // Buffering is important for performance

    let mut parser = EventReader::new_with_config(
        file,
        ParserConfig::new()
            .ignore_comments(true)
            .trim_whitespace(true)
            .whitespace_to_characters(true)
            .cdata_to_characters(true)
            .coalesce_characters(true),
    );

    let _ = dbg!(XDFElement::from_xml(&mut parser));
    Ok(())
}
