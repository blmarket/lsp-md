use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::{Location, Position, Range, Url};

use super::document::{BasicDocumentExt, DocumentAdapter};
use super::Encoder;

pub fn find_similar<D>(doc: &D, enc: &impl Encoder, pos: Position) -> Vec<Range>
where
    D: DocumentAdapter + BasicDocumentExt,
{
    let current_section_idx = doc.position_to_section(pos).unwrap();
    let current_section_embedding =
        enc.encode(doc.text(current_section_idx).unwrap()).unwrap();
    let mut sections: Vec<_> = doc
        .sections()
        .into_iter()
        .enumerate()
        .map(|(i, _)| {
            let embedding = enc.encode(doc.text(i).unwrap()).unwrap();
            let dist = embedding.dist(&current_section_embedding);
            (dist, i)
        })
        .collect();
    sections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    sections
        .into_iter()
        .take(11)
        .map(|(_, i)| doc.section_to_title_range(i).unwrap())
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct ScoredLocation {
    pub score: f32,
    pub title: String,
    pub location: Location,
}

pub fn find_similar2<D>(
    uri: Url,
    doc: &D,
    enc: &impl Encoder,
    pos: Position,
) -> Vec<ScoredLocation>
where
    D: DocumentAdapter + BasicDocumentExt,
{
    let current_section_idx = doc.position_to_section(pos).unwrap();
    let current_section_embedding =
        enc.encode(doc.text(current_section_idx).unwrap()).unwrap();
    let mut sections: Vec<_> = doc
        .sections()
        .into_iter()
        .enumerate()
        .map(|(i, _)| {
            let embedding = enc.encode(doc.text(i).unwrap()).unwrap();
            let dist = embedding.dist(&current_section_embedding);
            (dist, i)
        })
        .collect();
    sections.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    sections
        .into_iter()
        .take(11)
        .map(|(score, i)| {
            let range = doc.section_to_title_range(i).unwrap();
            ScoredLocation {
                score,
                title: doc.title(i).unwrap().to_string(),
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
    D: DocumentAdapter,
{
    (0..doc.sections().len())
        .map(|s| doc.section_to_title_range(s).unwrap())
        .collect()
}
