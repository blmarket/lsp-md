mod bert;
mod document_adapter;
mod document;
#[cfg(test)]
mod integration_tests;
mod similar_notes;
mod extract_keywords;

pub use bert::{BertModel, Encoder};
pub use document::Document;
pub use similar_notes::{find_similar, query_section_titles, ScoredLocation};
pub use extract_keywords::extract_keywords;
