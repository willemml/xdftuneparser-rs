#![feature(if_let_guard)]
#![feature(decl_macro)]

use std::fs::File;
use std::io::BufReader;

use xml::reader::EventReader;
use xml::ParserConfig;

use xdftuneparser::data_types::*;

/// needs to be broken out/improved/verified.
#[test]
fn parse_amb_xdf() {
    let file = File::open("tests/8E0909518AK_368072_NEF_STG_1v7.xdf").unwrap();
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

    let _ = XDFElement::from_xml(&mut parser).unwrap();
}
