use std::collections::BTreeMap;
use std::fmt;

/// Maximum descriptor size in bytes.
pub const MAX_DESCRIPTOR_BYTES: usize = 65_535;
/// Maximum number of normalized fields across all reports.
pub const MAX_FIELDS: usize = 4096;
/// Maximum expanded local usage list.
pub const MAX_USAGES: usize = 4096;
/// Maximum array usage entries retained across the entire layout.
pub const MAX_TOTAL_USAGES: usize = 65_536;
/// Maximum report payload length in bits.
pub const MAX_REPORT_BITS: u32 = 65_536;
/// Maximum nesting for both collections and global state.
pub const MAX_DEPTH: usize = 32;
/// Version of the normalized comparison and JSON semantics.
pub const CONTRACT_VERSION: u32 = 1;

/// Reason a descriptor cannot be interpreted under this contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// Truncated, inconsistent, or invalid encoding.
    Malformed,
    /// Legal or reserved semantics outside the supported profile.
    Unsupported,
    /// A documented resource limit was exceeded.
    Limit,
    /// The supplied report does not match the selected descriptor.
    Report,
}

/// Structured diagnostic. Never includes raw input or filesystem paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// Error category.
    pub kind: ErrorKind,
    /// Byte offset in the descriptor, hex text, or report as appropriate.
    pub offset: usize,
    /// Stable machine-readable diagnostic code.
    pub code: &'static str,
    /// Human-readable explanation.
    pub message: &'static str,
}

impl Error {
    pub(crate) fn new(
        kind: ErrorKind,
        offset: usize,
        code: &'static str,
        message: &'static str,
    ) -> Self {
        Self {
            kind,
            offset,
            code,
            message,
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}: {}", self.code, self.offset, self.message)
    }
}
impl std::error::Error for Error {}

/// HID report direction/category. Offsets are independent for each kind and ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReportKind {
    /// Device-to-host data.
    Input,
    /// Host-to-device data.
    Output,
    /// Control/configuration data.
    Feature,
}
impl fmt::Display for ReportKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Input => "input",
            Self::Output => "output",
            Self::Feature => "feature",
        })
    }
}

/// Report identity; ID 0 means no report-ID prefix.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReportKey {
    /// Report category.
    pub kind: ReportKind,
    /// Report ID, or zero for unnumbered reports.
    pub id: u8,
}

/// Collection route component. Sibling position disambiguates identical collections.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Collection {
    /// HID collection type.
    pub kind: u8,
    /// Fully qualified page/usage value.
    pub usage: u32,
    /// Zero-based position among sibling collections (including empty ones).
    pub sibling: u32,
}

/// Interpretation of a normalized field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Meaning {
    /// Constant padding; grouping and irrelevant global state are normalized away.
    Padding,
    /// One variable value with a fully qualified usage.
    Variable(u32),
    /// Array slots map logical values to this ordered usage list.
    Array(Vec<u32>),
}

/// Declared interpretation of data. Physical bounds are resolved per HID defaults.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Semantics {
    /// Main-item flags, including relative, null-state and volatility bits.
    pub flags: u32,
    /// Inclusive logical minimum.
    pub logical_min: i64,
    /// Inclusive logical maximum.
    pub logical_max: i64,
    /// Effective physical minimum.
    pub physical_min: i64,
    /// Effective physical maximum.
    pub physical_max: i64,
    /// Raw HID unit nibbles; no unit conversion is attempted.
    pub unit: u32,
    /// Decimal unit exponent.
    pub unit_exponent: i8,
    /// Route through enclosing collections.
    pub collections: Vec<Collection>,
}

/// One variable, one array group, or one contiguous padding run.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Field {
    /// Offset in payload bits, excluding the report-ID byte.
    pub bit_offset: u32,
    /// Slot width in bits; padding uses its total width.
    pub bit_size: u32,
    /// Slot count (one for variable and padding).
    pub count: u32,
    /// Value interpretation.
    pub meaning: Meaning,
    /// None for padding.
    pub semantics: Option<Semantics>,
    /// Byte offset of the Main item that introduced this field.
    pub source_offset: usize,
}
impl Field {
    pub(crate) fn contract_eq(&self, other: &Self) -> bool {
        self.bit_offset == other.bit_offset
            && self.bit_size == other.bit_size
            && self.count == other.count
            && self.meaning == other.meaning
            && self.semantics == other.semantics
    }
}

/// One fully parsed report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// Report identity.
    pub key: ReportKey,
    /// Payload length in bits, before final byte rounding.
    pub payload_bits: u32,
    /// Fields in payload order.
    pub fields: Vec<Field>,
}
impl Report {
    /// Bytes on the wire, including the ID byte when numbered.
    pub fn wire_bytes(&self) -> usize {
        self.payload_bits.div_ceil(8) as usize + usize::from(self.key.id != 0)
    }
}

/// Validated layout. Only the parser can construct it, preserving decode invariants.
#[derive(Clone, Debug)]
pub struct Layout {
    pub(crate) reports: BTreeMap<ReportKey, Report>,
}
impl Layout {
    /// Reports in stable kind/ID order. Returned data cannot mutate this layout.
    pub fn reports(&self) -> &BTreeMap<ReportKey, Report> {
        &self.reports
    }
    /// Whether all reports carry an ID byte.
    pub fn numbered(&self) -> bool {
        self.reports.keys().any(|key| key.id != 0)
    }
}
