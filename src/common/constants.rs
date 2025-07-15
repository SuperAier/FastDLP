/// Default sensitive data types supported by the system
pub const DEFAULT_SENSITIVE_TYPES: &[&str] = &[
    "PERSON",
    "ID_CARD",
    "TELEPHONE",
    "MOBILE_PHONE",
    "EMAIL",
    "LICENSE_PLATE",
    "BANK_CARD",
    "PASSPORT",
    "COMPANY_NAME",
    "SOCIAL_CREDIT_CODE",
    "IPV4",
    "IPV6",
    "MAC",
    "DOMAIN_NAME",
    "LOCATION",
    "POSTCODE",
    "DATE",
    "PASSWORD",
];

/// Type used for non-sensitive data
pub const OTHER_TYPE: &str = "OTHER";

/// Default threshold for classification
pub const DEFAULT_THRESHOLD: f64 = 0.8;

/// Maximum number of rows to process by default
pub const DEFAULT_MAX_ROWS: usize = 1000;

/// Regex patterns for common sensitive data types
pub const BUILTIN_PATTERNS: &[(&str, &str)] = &[
    ("ID_CARD", r"^\d{17}[\dXx]$"),
    ("MOBILE_PHONE", r"^1[3-9]\d{9}$"),
    ("TELEPHONE", r"^(\d{3,4}-)?\d{7,8}$"),
    ("EMAIL", r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$"),
    ("IPV4", r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$"),
    ("IPV6", r"^(?:[0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$"),
    ("MAC", r"^([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}$"),
    ("BANK_CARD", r"^\d{16,19}$"),
    ("POSTCODE", r"^\d{6}$"),
    ("LICENSE_PLATE", r"^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领][A-Z][0-9A-Z]{5}$"),
    ("PASSPORT", r"^[A-Z]\d{8}$"),
    ("SOCIAL_CREDIT_CODE", r"^[0-9A-HJ-NPQRTUWXY]{2}\d{6}[0-9A-HJ-NPQRTUWXY]{10}$"),
    ("DATE", r"^\d{4}[-/]\d{1,2}[-/]\d{1,2}$"),
];

/// Configuration for BPE (Byte Pair Encoding) in regex generation
pub const BPE_PAIR_PERCENT_THRESHOLD: f64 = 0.01;
pub const BPE_CHAR_PERCENT_THRESHOLD: f64 = 0.01;

/// Maximum depth for regex tree generation
pub const MAX_REGEX_TREE_DEPTH: usize = 10;

/// Default regex flavors
pub enum RegexFlavor {
    Python,
    Rust,
    JavaScript,
    PCRE,
} 