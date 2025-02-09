#![feature(if_let_guard)]

use std::io::{BufReader, Read};

use xml::{EventReader, ParserConfig};

pub mod data_types;
pub mod error;
pub mod parser;

pub fn parse_buffer<R: Read>(
    from: R,
) -> Result<Result<data_types::XDFElement, error::Error>, std::io::Error> {
    let file = BufReader::new(from); // Buffering is important for performance

    let mut parser = EventReader::new_with_config(
        file,
        ParserConfig::new()
            .ignore_comments(true)
            .trim_whitespace(true)
            .whitespace_to_characters(true)
            .cdata_to_characters(true)
            .coalesce_characters(true),
    );

    Ok(data_types::XDFElement::from_xml(&mut parser))
}
