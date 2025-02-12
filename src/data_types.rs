//! Datatypes found in TunerPro's XDF format
//! lots of info here: https://www.tunerpro.net/WebHelp/
//! Currently simplified to create IPO/MVP for editing 8E0909518AK-0003 XDFs/BINs, especially the Math/Conversion parts.
//!
//! Target files have a max size of 1MB, so everything fits in a 32bit address space.
//! Because of this, all addresses and sizes have a datatype of u32, in some cases these will be later converted to usize for use in Rust code, but that is not in scope for this module.

/// How values are shown to the user. These mappings to numbers may be wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum OutputType {
    Float = 0,
    Integer = 1,
    Hex = 2,
    String = 3,
}

/// Purpose unknown. Used in XDFHEADER
#[derive(Debug, Clone)]
pub struct Region {
    pub rtype: Option<u32>,
    pub startaddress: Option<u32>,
    pub size: Option<u32>,
    pub regionflags: Option<u32>,
}

/// Default configuration for items as defined in XDFHEADER
#[derive(Debug, Clone)]
pub struct Defaults {
    pub datasizeinbits: Option<u32>,
    pub sigdigits: Option<u32>,
    pub outputtype: Option<u32>,
    pub signed: Option<u32>,
    pub lsbfirst: Option<u32>,
    pub float: Option<u32>,
}

/// Data category for displaying XDF items
#[derive(Debug, Clone)]
pub struct Category {
    pub index: Option<u32>,
    pub name: Option<String>,
}

/// Header for XDF files, contains basic info such as origin of file.
/// Definition incomplete.
#[derive(Debug, Clone)]
pub struct XDFHeader {
    pub deftitle: Option<String>,
    pub description: Option<String>,
    pub fileversion: Option<String>,
    pub author: Option<String>,
    pub baseoffset: Option<u32>,
    pub defaults: Option<Defaults>,
    pub region: Option<Region>,
    pub flags: Option<u32>,
    // Could be array?
    pub category: Option<Category>,
}

/// Labels for XDFAXIS
#[derive(Debug, Clone)]
pub struct Label {
    pub index: Option<u32>,
    /// This may be wrong, but it seems these are only used for user defined values anyways, not calculations.
    pub value: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct EmbedInfo {
    /// Unknown Purpose
    /// linked axis: 3
    /// inplace definition with data location: 1
    pub etype: Option<u32>,
    /// Unique ID of the table describing the actual data
    /// Axis to use in the actual table seems to be the one with no `uniqueid` definition rather than `uniqueid="0x0"`
    pub linkobjid: Option<u32>,
}

/// Individual elements and their sub elements that can be found in an XML formatted XDF filke
#[derive(Debug, Clone)]
pub enum XDFElement {
    End(String),
    Title(String),
    Description(String),
    XDFHeader(XDFHeader),
    Region(Region),
    FileVersion(String),
    Author(String),
    Defaults(Defaults),
    Category(Category),
    XDFFormat(XDFFormat),
    Flags(u32),
    Label(Label),
    BaseOffset(u32),
    /// Purpose Unknown
    /// Found in XDFAXIS and XDFCONSTANT
    /// Contains "index"
    /// example (from KFMIRL X axis): `<DALINK index="0" />`
    DALink(u32),
    /// Exact Purpose Unknown
    /// Found in XDFCONSTANT
    /// example: `<datatype>0</datatype>`
    DataType(u32),
    /// Exact Purpose Unknown
    /// Found in XDFCONSTANT
    /// example: `<unittype>0</unittype>`
    UnitType(u32),
    /// Units of an axis or constant
    /// example: `<units>RPM</units>`
    Units(String),
    /// Minimum possible value for an axis
    /// Found in XDFAXIS
    /// example: `<min>0.000000</min>`
    Min(f32),
    /// Maximum possible value for an axis
    /// Found in XDFAXIS
    /// example: `<max>255.000000</max>`
    Max(f32),
    /// Display format for the data
    /// Guesses:
    ///     1: Float
    ///     2: Integer
    ///     3: Hex
    ///     4: String
    /// Found in XDFAXIS
    /// example: `<outputtype>1</outputtype>`
    OutputType(u32),
    /// Number of decimal places to display in UI
    /// Found in XDFAXIS
    /// example: `<outputtype>1</outputtype>`
    DecimalPl(u32),
    /// Describes where and how the data is stored in a bin
    /// Found in XDFAXIS and XDFCONSTANT
    /// example (from an XDFAXIS): `<EMBEDDEDDATA mmedtypeflags="0x02" mmedaddress="0x1EF7E" mmedelementsizebits="16" mmedrowcount="16" mmedmajorstridebits="0" mmedminorstridebits="0" />`
    EmbeddedData(EmbeddedData),
    /// Usage Unknown, likely refers to an index in an array of categories to determine where to put the item in a tree view.
    /// Found in XDFTABLE and XDFCONSTANT
    /// example (from XDFCONSTANT): `<CATEGORYMEM index="0" category="27" />`
    CategoryMem(CategoryMem),
    /// Transformation to perform when reading from or writing to memory location before displaying
    /// e.g. KRKTE is stored as 622 in bin file, but is displayed as 0.1039
    /// example (from KRKTE table Z axis): ```xml
    /// <MATH equation="0.000000+X*0.000167">
    ///     <VAR id="X" />
    /// </MATH>
    /// ```
    Math(Math),
    /// Variables used in MATH, there seems to be only one variable (usually X) most of the time
    /// example: `<VAR id="X" />`
    Var(String),
    /// Number of items in an axis, appears to be used instead of column or row count from EMBEDDEDDATA
    /// example: `<indexcount>5</indexcount>`
    IndexCount(u32),
    /// Indicates special handling of data embedding somehow.
    /// example (from TVUB): `<embedinfo type="3" linkobjid="0x14DA9" />`
    EmbedInfo(EmbedInfo),
    /// Single value constant, unsure of practical difference between this and a 0x0x1 table.
    /// example: ```xml
    ///   <XDFCONSTANT uniqueid="0x3BFE">
    ///     <title>CDTES</title>
    ///     <description>Codeword: turn off tank venting diagnosis (EURO-Coding), CD..=0 -&gt;no Dia</description>
    ///     <CATEGORYMEM index="0" category="27" />
    ///     <EMBEDDEDDATA mmedaddress="0x181B2" mmedelementsizebits="8" mmedmajorstridebits="0" mmedminorstridebits="0" />
    ///     <datatype>0</datatype>
    ///     <unittype>0</unittype>
    ///     <DALINK index="0" />
    ///     <MATH equation="X">
    ///       <VAR id="X" />
    ///     </MATH>
    ///   </XDFCONSTANT>
    /// ```
    XDFConstant(XDFConstant),
    /// Axis definition for a table, generally contains a series of labels (non stored values) or a data location (values stored in bin)
    /// This is likely only used for display purposes.
    /// Labels can be stored internally (in the BIN) or externally (defined in the XDF)
    /// example (from KFMIRL): ```xml
    /// <XDFAXIS id="z">
    ///   <EMBEDDEDDATA mmedtypeflags="0x02" mmedaddress="0x1EF7E" mmedelementsizebits="16" mmedrowcount="16" mmedmajorstridebits="0" mmedminorstridebits="0" />
    ///   <decimalpl>2</decimalpl>
    ///   <min>0.000000</min>
    ///   <max>255.000000</max>
    ///   <outputtype>1</outputtype>
    ///   <MATH equation="X/4">
    ///     <VAR id="X" />
    ///   </MATH>
    /// </XDFAXIS>
    /// ```
    XDFAxis(XDFAxis),
    /// Table, contains multiple (three seems to be common) axis
    /// For some reason, instead of having multiple editeable axis, some tables will have a sepearate linked table defining an editable (stored) axis.
    /// TVUB is an example of this with its axis data being stored in TVUB_AXIS.
    /// example: ```xml
    ///   <XDFTABLE uniqueid="0x14DAE" flags="0x0">
    ///     <title>TVUB</title>
    ///     <XDFAXIS id="x" uniqueid="0x0">
    ///       <EMBEDDEDDATA mmedelementsizebits="16" mmedmajorstridebits="-32" mmedminorstridebits="0" />
    ///       <indexcount>1</indexcount>
    ///       <datatype>0</datatype>
    ///       <unittype>0</unittype>
    ///       <DALINK index="0" />
    ///       <LABEL index="0" value="0.00" />
    ///       <MATH equation="X">
    ///         <VAR id="X" />
    ///       </MATH>
    ///     </XDFAXIS>
    ///     <XDFAXIS id="y" uniqueid="0x0">
    ///       <EMBEDDEDDATA mmedelementsizebits="16" mmedmajorstridebits="-32" mmedminorstridebits="0" />
    ///       <indexcount>5</indexcount>
    ///       <embedinfo type="3" linkobjid="0x14DA9" />
    ///       <datatype>0</datatype>
    ///       <unittype>0</unittype>
    ///       <DALINK index="0" />
    ///       <MATH equation="X">
    ///         <VAR id="X" />
    ///       </MATH>
    ///     </XDFAXIS>
    ///     <XDFAXIS id="z">
    ///       <EMBEDDEDDATA mmedtypeflags="0x02" mmedaddress="0x14DAE" mmedelementsizebits="16" mmedrowcount="5" mmedmajorstridebits="0" mmedminorstridebits="0" />
    ///       <decimalpl>2</decimalpl>
    ///       <min>0.000000</min>
    ///       <max>255.000000</max>
    ///       <outputtype>1</outputtype>
    ///       <MATH equation="0.000000+X*0.002667">
    ///         <VAR id="X" />
    ///       </MATH>
    ///     </XDFAXIS>
    ///   </XDFTABLE>
    /// ```
    XDFTable(XDFTable),
}

/// Operations to perform on a value before displaying or writing it.
/// Seems ususally have a single variable (X) which is the value stored in the bin.
/// There is generally only a factor and or a constant to be applied.
#[derive(Debug, Default, Clone)]
pub struct Math {
    /// Variables used in equation, usually just X
    pub vars: Vec<String>,
    /// Expression applied to stored value (using variables defined in `vars`)
    /// Can likely be simplified to a constant and a factor.
    pub expression: Option<String>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CategoryMem {
    pub index: Option<u32>,
    pub category: Option<u32>,
}

/// Describes how and where the data is stored in the bin file, if mmedaddress is undefined it is not stored or read from the bin.
/// When rowcount and colcount are both greater than 1, each row is written sequentially.
/// For example with 8 rows of 16 columns, you would write each of the 16 column values for a row before writing the next row.
/// Data is stored in a one dimensional array :)
#[derive(Debug, Default, Clone, Copy)]
pub struct EmbeddedData {
    /// Base address (relative to start of file) of data
    pub mmedaddress: Option<u32>,
    /// Size in bits of each element
    pub mmedelementsizebits: Option<u32>,
    pub mmedmajorstridebits: Option<i32>, // ?
    pub mmedminorstridebits: Option<i32>, // ?
    /// Seems to only be present when in XDFCONSTANT
    pub mmedtypeflags: Option<u32>, // ?
    /// Number of rows
    pub mmedrowcount: Option<u32>,
    /// Number of columns
    pub mmedcolcount: Option<u32>,
}

/// Single value constant, unsure of practical difference between this and a 0x0x1 table.
#[derive(Debug, Default, Clone)]
pub struct XDFConstant {
    pub title: Option<String>,
    pub description: Option<String>,
    pub catmem: Option<CategoryMem>, // ?
    pub uid: Option<String>,         // uniqueid, doesnt actually seem to be unique
    pub embedded_data: Option<EmbeddedData>,
    pub decimalplaces: Option<u32>,
    pub datatype: Option<u32>,   // unknown
    pub unittype: Option<u32>,   // unknown
    pub outputtype: Option<u32>, // unknown
    pub unit: Option<String>,
    pub dalink_index: Option<u32>, // unknown
    pub math: Option<Math>,
}

/// Axis definition for a table, generally contains a series of labels (non stored values) or a data location (values stored in bin)
/// This is likely only used for display purposes.
#[derive(Debug, Default, Clone)]
pub struct XDFAxis {
    pub id: Option<String>, // name
    pub uid: Option<u32>,   // uniqueid, doesnt actually seem to be unique
    pub embeddeddata: Option<EmbeddedData>,
    pub datatype: Option<u32>,
    pub unittype: Option<u32>,
    pub dalink_index: Option<u32>,
    pub math: Option<Math>,
    pub count: Option<u32>, // how many items in axis
    pub labels: Vec<Label>,
    // above this line probably required
    pub min: Option<f32>,           // min value
    pub max: Option<f32>,           // max value
    pub outputtype: Option<u32>,    // unknown
    pub decimalplaces: Option<u32>, // how many dceimal places to display, doenst seem to effect output
    pub unit: Option<String>,
    pub embedinfo: Option<EmbedInfo>,
}

/// Table, contains multiple (three seems to be common) axis
/// For some reason, instead of having multiple editeable axis, some tables will have a sepearate linked table defining an editable (stored) axis.
/// TVUB is an example of this with its axis data being stored in TVUB_AXIS.
#[derive(Debug, Default, Clone)]
pub struct XDFTable {
    pub title: Option<String>, // obvious
    pub uid: Option<u32>,      // bitcount?
    pub flags: Option<u32>,    // bitcount? purpose unknown
    pub catmem: Option<CategoryMem>,
    pub description: Option<String>,
    pub axis: Vec<XDFAxis>, // duh
}

/// A complete XDF file
#[derive(Debug, Clone)]
pub struct XDFFormat {
    pub version: Option<String>,
    pub tables: Vec<XDFTable>,
    pub constants: Vec<XDFConstant>,
    pub header: Option<XDFHeader>,
}
