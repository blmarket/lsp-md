use std::borrow::Cow;

use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

use super::document::DocumentExt;
use super::document_adapter::DocumentLsp;
use super::Encoder;

#[derive(Debug, Serialize, Deserialize)]
pub struct ScoredLocation {
    pub score: f32,
    pub title: String,
    pub location: Location,
}

pub fn find_similar<'a, D>(
    uri: Url,
    doc: &'a D,
    enc: &impl Encoder,
    pos: &Position,
) -> Vec<ScoredLocation>
where
    D: DocumentLsp + DocumentExt<'a>,
{
    let current_section_idx = doc.position_to_section(pos).unwrap();
    let text: Cow<'a, str> = DocumentExt::text(doc, current_section_idx).expect("should have section").into();
    let current_section_embedding =
        enc.encode(text).unwrap();
    let mut sections: Vec<_> = doc
        .sections()
        .into_iter()
        .enumerate()
        .map(|(i, _)| {
            let t2: Cow<'a, str> = DocumentExt::text(doc, i).expect("should have section").into();
            let embedding = enc .encode(t2) .unwrap();
            let dist = embedding.cos(&current_section_embedding);
            (dist, i)
        })
        .collect();
    sections.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    sections
        .into_iter()
        .take(11)
        .map(|(score, i)| {
            let title = DocumentExt::title(doc, i).expect("should have section");
            let range = doc.section_to_title_range(i).unwrap();
            ScoredLocation {
                score,
                title: title.into(),
                location: Location {
                    uri: uri.clone(),
                    range,
                },
            }
        })
        .collect()
}

pub fn query_section_titles<D>(doc: &D) -> Vec<Range>
where
    D: DocumentLsp,
{
    (0..doc.sections().len())
        .map(|s| doc.section_to_title_range(s).unwrap())
        .collect()
}
