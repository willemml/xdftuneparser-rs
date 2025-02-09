use xml::reader::XmlEvent;

use crate::data_types::XDFElement;

#[derive(Debug)]
pub enum Error {
    MissingItem,
    BadValue,
    UnknownType,
    UnexpectedElement(XDFElement),
    UnexpectedEvent(XmlEvent),
    LeftoverData,
    XmlError(xml::reader::Error),
}

impl From<xml::reader::Error> for Error {
    fn from(value: xml::reader::Error) -> Self {
        Self::XmlError(value)
    }
}
