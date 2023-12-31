mod bert;
mod document;
mod document_adapter;
mod extract_keywords;
mod find_by_keyword;
mod format;
#[cfg(test)]
mod integration_tests;
mod similar_notes;

pub use bert::{BertModel, Encoder};
pub use document::Document;
pub use extract_keywords::extract_keywords;
pub use find_by_keyword::find_by_keyword;
pub use format::{LspRangeFormat, Formatter as CodeFormatter};
pub use similar_notes::{find_similar, query_section_titles, ScoredLocation};
