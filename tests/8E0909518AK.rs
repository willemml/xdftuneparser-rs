#![feature(if_let_guard)]
#![feature(decl_macro)]

use std::fs::File;
use xdftuneparser::parse_buffer;

/// needs to be broken out/improved/verified.
#[test]
fn parse_amb_xdf() {
    let file = File::open("tests/8E0909518AK_368072_NEF_STG_1v7.xdf").unwrap();
    parse_buffer(file).unwrap().unwrap();
}
