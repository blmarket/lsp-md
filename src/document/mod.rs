mod bert;
mod document;
#[cfg(test)]
mod integration_tests;
mod similar_notes;

pub use bert::{Encoder, BertModels};
pub use document::Document;
pub use similar_notes::{
    find_similar, find_similar2, query_section_titles, ScoredLocation,
};
