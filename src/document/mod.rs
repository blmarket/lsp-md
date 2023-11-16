mod bert;
mod document;
mod document_adapter;
mod extract_keywords;
mod find_using_keyword;
#[cfg(test)]
mod integration_tests;
mod similar_notes;

pub use bert::{BertModel, Encoder};
pub use document::Document;
pub use extract_keywords::extract_keywords;
pub use find_using_keyword::find_using_keyword;
pub use similar_notes::{find_similar, query_section_titles, ScoredLocation};
