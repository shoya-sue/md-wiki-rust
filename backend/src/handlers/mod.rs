mod document;
mod metadata;

pub use document::{get_document, save_document, list_documents, search_documents, get_document_history, get_document_version};
pub use metadata::{get_document_metadata, update_document_metadata, get_all_tags, search_documents_by_tag, get_recent_documents}; 