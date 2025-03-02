//! Parser functions for XDF, general flow is a loop that grabs the next
//! This is likely not the best way of doing this, but it was fairly easy to write as a MVP.
//! Should be rewritten later.

use std::str::FromStr;

use xml::{attribute::OwnedAttribute, name::OwnedName, reader::XmlEvent, EventReader};

use crate::{data_types::*, error::Error};

/// Reads a string from an XML characters event, used to parse data stored within an element rather than as an attribute.
/// e.g. `<title>DATAHERE</title>`
/// If there is no data present an empty string is returned.
fn from_chars<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<String, Error> {
    let next = parser.next()?;
    let mut end = false;
    let res = if let xml::reader::XmlEvent::Characters(chars) = next {
        Ok(chars)
    } else if let XmlEvent::EndElement { name: _ } = next {
        end = true;
        Ok(String::new())
    } else {
        Err(crate::error::Error::UnexpectedEvent(next))
    };
    if !end {
        let _ = parser.next()?;
    }
    res
}

/// Parses an integer, stored in base10 or base16 ASCII.
/// If string starts with `0x` assumes base16, otherwise base10.
fn parse_int(from: &str) -> Result<u32, Error> {
    let stripped = from.strip_prefix("0x");
    if let Some(hex) = stripped {
        u32::from_str_radix(hex, 16).map_err(|_| panic!("Error::BadValue"))
    } else {
        from.parse().map_err(|_| panic!("Error::BadValue"))
    }
}

/// Convenience function to use `parse_int` on the output of `get_attr`
fn int_attr(attrs: &Vec<OwnedAttribute>, name: &str) -> Result<u32, Error> {
    parse_int(&get_attr(attrs, name)?)
}

/// Convenience function to parse the output of `from_chars`
fn parse_chars<R: std::io::Read, T: FromStr>(parser: &mut EventReader<R>) -> Result<T, Error> {
    from_chars(parser)?.parse().map_err(|_| Error::BadValue)
}

/// Gets the string value of a named attribute
fn get_attr(attrs: &Vec<OwnedAttribute>, name: &str) -> Result<String, Error> {
    for attr in attrs {
        if attr.name.local_name == name {
            return Ok(attr.value.clone());
        }
    }
    return Err(Error::MissingItem);
}

/// Convenience function to parse the output of `get_attr`, use `int_attr` instead when possible.
fn get_attr_parse<T: FromStr>(attrs: &Vec<OwnedAttribute>, name: &str) -> Result<T, Error> {
    get_attr(attrs, name)?.parse().map_err(|_| Error::BadValue)
}

/// Creates a function that builds an object by looping over an XmlReader.
/// Has three ways of defining a field, either from another type of known XMLElement that can be parsed by `XDFElement::from_xml`.
/// Or, a list of elements of the same type.
/// Or, an element that requires an external function to parse, usually stored in an attribute.
/// The generated function consumes any end of element events.
macro_rules! build_obj {
    ($parser:ident,$name:expr,$type:ident,[$($fieldname:ident ; $fieldsource:ident),*]) => {build_obj!($parser, $name, $type, [$($fieldname;$fieldsource),*],[],[])};
    ($parser:ident,$name:expr,$type:ident,[$($fieldname:ident ; $fieldsource:ident),*], [$($fieldn:ident ; $fieldcalc:block),*],[$($vfname:ident;$vfsource:ident),*]) => {
        {
            $(
                let mut $fieldname = None;
            )*
            $(
                let mut $vfname = Vec::new();
            )*

            loop {
                match XDFElement::from_xml($parser)? {
                    $(
                        XDFElement::$fieldsource(v) => $fieldname = Some(v),
                    )*
                    $(
                        XDFElement::$vfsource(v) => $vfname.push(v),
                    )*
                    XDFElement::End(name) => if &name == $name {
                        break;
                    } else {
                        continue;
                    }
                    e => return Err(Error::UnexpectedElement(e)),
                }
                }

            XDFElement::$type($type {
                $(
                    $fieldname: $fieldname,
                )*
                $(
                    $vfname: $vfname,
                )*
                $(
                    $fieldn: $fieldcalc,
                )*
            })
        }
    };
    ($parser:ident,$type:ident,[$($fname:ident:$fsource:expr,)*]) => {{
        let r = XDFElement::$type($type {
            $(
                $fname: $fsource,
            )*
        });
        let next = XDFElement::from_xml($parser)?;
        if let XDFElement::End(_) = next {
            r
        } else {
            Err(Error::UnexpectedElement(next))?
        }
    }}
}

impl XDFElement {
    /// Parses either an entire XDF document, or a single element (including all it's children)
    pub fn from_xml<R: std::io::Read>(parser: &mut EventReader<R>) -> Result<Self, Error> {
        let next;
        loop {
            let current = parser.next()?;
            if let XmlEvent::StartDocument { .. } = &current {
                continue;
            }
            next = Some(match current {
                xml::reader::XmlEvent::StartElement {
                    name, attributes, ..
                } => match name.local_name.to_lowercase().as_str() {
                    "title" | "deftitle" => Self::Title(from_chars(parser)?),
                    "description" => Self::Description(from_chars(parser)?),
                    "units" => Self::Units(from_chars(parser)?),
                    "author" => Self::Author(from_chars(parser)?),
                    "fileversion" => Self::FileVersion(from_chars(parser)?),
                    "indexcount" => Self::IndexCount(parse_chars(parser)?),
                    "datatype" => Self::DataType(parse_chars(parser)?),
                    "unittype" => Self::UnitType(parse_chars(parser)?),
                    "outputtype" => Self::OutputType(parse_int(&from_chars(parser)?)?),
                    "decimalpl" => Self::DecimalPl(parse_chars(parser)?),
                    "flags" => Self::Flags(parse_int(&from_chars(parser)?)?),
                    "min" => Self::Min(parse_chars(parser)?),
                    "max" => Self::Max(parse_chars(parser)?),
                    "baseoffset" => {
                        if let Ok(offset) = get_attr_parse(&attributes, "offset") {
                            Self::BaseOffset(offset)
                        } else {
                            Self::BaseOffset(parse_int(&from_chars(parser)?)?)
                        }
                    }
                    "dalink" => {
                        let r = Self::DALink(get_attr_parse(&attributes, "index")?);
                        parser.next()?;
                        r
                    }
                    "var" => {
                        let r = Self::Var(get_attr_parse(&attributes, "id")?);
                        parser.next()?;
                        r
                    }
                    "embedinfo" => build_obj!(parser, EmbedInfo, [
                        etype: int_attr(&attributes, "type").ok(),
                        linkobjid: int_attr(&attributes, "linkobjid").ok(),
                    ]),
                    "defaults" => build_obj!(parser, Defaults, [
                        datasizeinbits: int_attr(&attributes, "datasizeinbits").ok(),
                        sigdigits: int_attr(&attributes, "sigdigits").ok(),
                        outputtype: int_attr(&attributes, "outputtype").ok(),
                        signed: int_attr(&attributes, "signed").ok(),
                        lsbfirst: int_attr(&attributes, "lsbfirst").ok(),
                        float: int_attr(&attributes, "float").ok(),
                    ]),
                    "category" => build_obj!(parser, Category,[
                        index: int_attr(&attributes, "index").ok(),
                        name: get_attr(&attributes, "name").ok(),
                    ]),
                    "region" => build_obj!(parser, Region,[
                        rtype: int_attr(&attributes, "type").ok(),
                        startaddress: int_attr(&attributes, "startaddress").ok(),
                        size: int_attr(&attributes, "size").ok(),
                        regionflags: int_attr(&attributes, "regionflags").ok(),
                    ]),
                    "categorymem" => build_obj!(parser, CategoryMem, [
                        index: get_attr_parse(&attributes, "index").ok(),
                        category: get_attr_parse(&attributes, "category").ok(),
                    ]),
                    "embeddeddata" => build_obj!(parser, EmbeddedData, [
                        mmedaddress: int_attr(&attributes, "mmedaddress").ok(),
                        mmedelementsizebits: get_attr_parse(&attributes, "mmedelementsizebits").ok(),
                        mmedmajorstridebits: get_attr_parse(&attributes, "mmedmajorstridebits").ok(),
                        mmedminorstridebits: get_attr_parse(&attributes, "mmedminorstridebits").ok(),
                        mmedtypeflags: get_attr_parse(&attributes, "mmedtypeflags").ok(),
                        mmedrowcount: get_attr_parse(&attributes, "mmedrowcount").ok(),
                        mmedcolcount: get_attr_parse(&attributes, "mmedcolcount").ok(),
                    ]),
                    "label" => build_obj!(parser, Label, [
                        index: get_attr_parse(&attributes, "index").ok(),
                        value: get_attr_parse(&attributes, "value").ok(),
                    ]),
                    "math" => build_obj!(
                        parser,
                        "math",
                        Math,
                        [],
                        [expression; { get_attr_parse(&attributes, "equation").ok() }],
                        [vars; Var]
                    ),
                    "xdfformat" => build_obj!(parser, "xdfformat", XDFFormat, [
                        header; XDFHeader
                    ], [
                        version; {get_attr_parse(&attributes, "version").ok()}
                    ], [
                        constants; XDFConstant,
                        tables; XDFTable
                    ]),
                    "xdftable" => build_obj!(parser, "xdftable", XDFTable, [
                        title; Title,
                        flags; Flags,
                        description; Description
                    ],[
                        uid; {int_attr(&attributes, "uniqueid").ok()}
                    ],[
                        catmem; CategoryMem,
                        axis; XDFAxis
                    ]),
                    "xdfheader" => build_obj!(parser, "xdfheader", XDFHeader, [
                        deftitle; Title,
                        description; Description,
                        baseoffset; BaseOffset,
                        defaults; Defaults,
                        region; Region,
                        flags; Flags,
                        fileversion; FileVersion,
                        author; Author
                    ],[],[
                        category; Category
                    ]),
                    "xdfaxis" => build_obj!(parser, "xdfaxis", XDFAxis, [
                        embeddeddata; EmbeddedData,
                        min; Min,
                        max; Max,
                        outputtype; OutputType,
                        datatype; DataType,
                        unittype; UnitType,
                        dalink_index; DALink,
                        count; IndexCount,
                        decimalplaces; DecimalPl,
                        math; Math,
                        unit; Units,
                        embedinfo; EmbedInfo
                    ], [
                        id; {get_attr(&attributes, "id").ok()},
                        uid; {int_attr(&attributes, "uniqueid").ok()}
                    ],[
                        labels; Label
                    ]),
                    "xdfconstant" => build_obj!(parser, "xdfconstant", XDFConstant, [
                        embedded_data; EmbeddedData,
                        title; Title,
                        description; Description,
                        outputtype; OutputType,
                        datatype; DataType,
                        decimalplaces; DecimalPl,
                        unittype; UnitType,
                        unit; Units,
                        math; Math,
                        dalink_index; DALink
                    ],[
                        uid; {get_attr_parse(&attributes, "uniqueid").ok()}
                    ],[
                        catmem; CategoryMem
                    ]),
                    "xdfpatch" | "xdfflag" | "xdfchecksum" => {
                        loop {
                            let event = parser.next()?;
                            let end1 = XmlEvent::EndElement {
                                name: OwnedName::local("XDFPATCH"),
                            };
                            let end2 = XmlEvent::EndElement {
                                name: OwnedName::local("XDFFLAG"),
                            };
                            let end3 = XmlEvent::EndElement {
                                name: OwnedName::local("XDFCHECKSUM"),
                            };
                            if event == end1 || event == end2 || event == end3 {
                                break;
                            }
                        }
                        continue;
                    }
                    u => {
                        dbg!(u);
                        return Err(Error::UnknownType);
                    }
                },
                XmlEvent::EndElement { name } => Self::End(name.local_name.to_lowercase()),
                e => return Err(Error::UnexpectedEvent(e)),
            });
            break;
        }

        Ok(next.unwrap())
    }
}
