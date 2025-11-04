use serde::{Deserialize, Serialize};

/// Supported document formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentFormat {
    Pdf,
    Epub,
    Cbz,
    Cbr,
    Txt,
    Markdown,
}

impl DocumentFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "pdf" => Some(Self::Pdf),
            "epub" => Some(Self::Epub),
            "cbz" => Some(Self::Cbz),
            "cbr" => Some(Self::Cbr),
            "txt" => Some(Self::Txt),
            "md" | "markdown" => Some(Self::Markdown),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Epub => "epub",
            Self::Cbz => "cbz",
            Self::Cbr => "cbr",
            Self::Txt => "txt",
            Self::Markdown => "markdown",
        }
    }
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub subject: Option<String>,
    pub language: Option<String>,
    pub created_at: Option<i64>,
    pub modified_at: Option<i64>,
    pub page_count: Option<usize>,
}

impl Default for Metadata {
    fn default() -> Self {
        Self {
            title: String::from("Untitled"),
            author: None,
            publisher: None,
            subject: None,
            language: None,
            created_at: None,
            modified_at: None,
            page_count: None,
        }
    }
}
