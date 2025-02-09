//! Parser functions for XDF, general flow is a loop that grabs the next
//! This is likely not the best way of doing this, but it was fairly easy to write as a MVP.
//! Should be rewritten later.

use std::str::FromStr;

use xml::{attribute::OwnedAttribute, reader::XmlEvent, EventReader};

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
        u32::from_str_radix(hex, 16).map_err(|_| Error::BadValue)
    } else {
        from.parse().map_err(|_| Error::BadValue)
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
                } => match name.local_name.as_str() {
                    "title" | "deftitle" => Self::Title(from_chars(parser)?),
                    "description" => Self::Description(from_chars(parser)?),
                    "units" => Self::Units(from_chars(parser)?),
                    "indexcount" => Self::IndexCount(parse_chars(parser)?),
                    "datatype" => Self::DataType(parse_chars(parser)?),
                    "unittype" => Self::UnitType(parse_chars(parser)?),
                    "outputtype" => Self::OutputType(parse_int(&from_chars(parser)?)?),
                    "decimalpl" => Self::DecimalPl(parse_chars(parser)?),
                    "flags" => Self::Flags(parse_int(&from_chars(parser)?)?),
                    "min" => Self::Min(parse_chars(parser)?),
                    "max" => Self::Max(parse_chars(parser)?),
                    "baseoffset" => Self::BaseOffset(parse_int(&from_chars(parser)?)?),
                    "DALINK" => {
                        let r = Self::DALink(get_attr_parse(&attributes, "index")?);
                        parser.next()?;
                        r
                    }
                    "VAR" => {
                        let r = Self::Var(get_attr_parse(&attributes, "id")?);
                        parser.next()?;
                        r
                    }
                    "embedinfo" => build_obj!(parser, EmbedInfo, [
                        etype: int_attr(&attributes, "type").ok(),
                        linkobjid: int_attr(&attributes, "linkobjid").ok(),
                    ]),
                    "DEFAULTS" => build_obj!(parser, Defaults, [
                        datasizeinbits: int_attr(&attributes, "datasizeinbits").ok(),
                        sigdigits: int_attr(&attributes, "sigdigits").ok(),
                        outputtype: int_attr(&attributes, "outputtype").ok(),
                        signed: int_attr(&attributes, "signed").ok(),
                        lsbfirst: int_attr(&attributes, "lsbfirst").ok(),
                        float: int_attr(&attributes, "float").ok(),
                    ]),
                    "CATEGORY" => build_obj!(parser, Category,[
                        index: int_attr(&attributes, "index").ok(),
                        name: get_attr(&attributes, "name").ok(),
                    ]),
                    "REGION" => build_obj!(parser, Region,[
                        rtype: int_attr(&attributes, "type").ok(),
                        startaddress: int_attr(&attributes, "startaddress").ok(),
                        size: int_attr(&attributes, "size").ok(),
                        regionflags: int_attr(&attributes, "regionflags").ok(),
                    ]),
                    "CATEGORYMEM" => build_obj!(parser, CategoryMem, [
                        index: get_attr_parse(&attributes, "index").ok(),
                        category: get_attr_parse(&attributes, "category").ok(),
                    ]),
                    "EMBEDDEDDATA" => build_obj!(parser, EmbeddedData, [
                        mmedaddress: int_attr(&attributes, "mmedaddress").ok(),
                        mmedelementsizebits: get_attr_parse(&attributes, "mmedelementsizebits").ok(),
                        mmedmajorstridebits: get_attr_parse(&attributes, "mmedmajorstridebits").ok(),
                        mmedminorstridebits: get_attr_parse(&attributes, "mmedminorstridebits").ok(),
                        mmedtypeflags: get_attr_parse(&attributes, "mmedtypeflags").ok(),
                        mmedrowcount: get_attr_parse(&attributes, "mmedrowcount").ok(),
                        mmedcolcount: get_attr_parse(&attributes, "mmedcolcount").ok(),
                    ]),
                    "LABEL" => build_obj!(parser, Label, [
                        index: get_attr_parse(&attributes, "index").ok(),
                        value: get_attr_parse(&attributes, "value").ok(),
                    ]),
                    "MATH" => build_obj!(
                        parser,
                        "MATH",
                        Math,
                        [],
                        [expression; { get_attr_parse(&attributes, "equation").ok() }],
                        [vars; Var]
                    ),
                    "XDFFORMAT" => build_obj!(parser, "XDFFORMAT", XDFFormat, [
                        header; XDFHeader
                    ], [
                        version; {get_attr_parse(&attributes, "version").ok()}
                    ], [
                        constants; XDFConstant,
                        tables; XDFTable
                    ]),
                    "XDFTABLE" => build_obj!(parser, "XDFTABLE", XDFTable, [
                        title; Title,
                        flags; Flags,
                        description; Description,
                        catmem; CategoryMem
                    ],[
                        uid; {get_attr_parse(&attributes, "uniqueid").ok()}
                    ],[
                        axis; XDFAxis
                    ]),
                    "XDFHEADER" => build_obj!(parser, "XDFHEADER", XDFHeader, [
                        category; Category,
                        deftitle; Title,
                        description; Description,
                        baseoffset; BaseOffset,
                        defaults; Defaults,
                        region; Region,
                        flags; Flags
                    ]),
                    "XDFAXIS" => build_obj!(parser, "XDFAXIS", XDFAxis, [
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
                        uid; {get_attr(&attributes, "uniqueid").ok()}
                    ],[
                        labels; Label
                    ]),
                    "XDFCONSTANT" => build_obj!(parser, "XDFCONSTANT", XDFConstant, [
                        embedded_data; EmbeddedData,
                        title; Title,
                        description; Description,
                        catmem; CategoryMem,
                        datatype; DataType,
                        unittype; UnitType,
                        unit; Units,
                        math; Math,
                        dalink_index; DALink
                    ],[
                        uid; {get_attr_parse(&attributes, "uniqueid").ok()}
                    ],[]),
                    u => {
                        dbg!(u);
                        return Err(Error::UnknownType);
                    }
                },
                XmlEvent::EndElement { name } => Self::End(name.local_name),
                e => return Err(Error::UnexpectedEvent(e)),
            });
            break;
        }

        Ok(next.unwrap())
    }
}
