mod bert;
mod document;
#[cfg(test)]
mod integration_tests;
mod similar_notes;

pub use bert::{BertModel, Encoder};
pub use document::Document;
pub use similar_notes::{
    find_similar2, query_section_titles, ScoredLocation,
};
