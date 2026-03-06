//! OFD main entry structure (OFD.xml).

/// Document metadata (CT_DocInfo).
#[derive(Debug, Clone, Default)]
pub struct DocInfo {
    /// Unique document ID (UUID, 32 chars).
    pub doc_id: String,
    /// Document title.
    pub title: Option<String>,
    /// Document author.
    pub author: Option<String>,
    /// Creator application name.
    pub creator: Option<String>,
    /// Creator application version.
    pub creator_version: Option<String>,
    /// Creation date (ISO format).
    pub creation_date: Option<String>,
}
