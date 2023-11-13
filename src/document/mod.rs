mod bert;
mod document;
mod embedding;
#[cfg(test)]
mod integration_tests;
mod keywords;
mod similar_notes;

pub use bert::Model;
pub use document::Document;
pub use similar_notes::{
    find_similar, find_similar2, query_section_titles, ScoredLocation,
};
