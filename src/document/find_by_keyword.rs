use std::borrow::Cow;

use tower_lsp::lsp_types::{Location, Url};

use super::document::DocumentExt;
use super::document_adapter::DocumentLsp;
use super::{Encoder, ScoredLocation};

pub fn find_by_keyword<'a, D>(
    uri: Url,
    model: &impl Encoder,
    doc: &'a D,
    keyword: &str,
) -> Vec<ScoredLocation>
where
    D: DocumentLsp + DocumentExt<'a>,
{
    let word_embedding =
        model.encode(keyword).expect("should calculate embedding");
    let mut candidates = doc
        .sections()
        .into_iter()
        .enumerate()
        .map(|(i, _)| {
            // FIXME: use batch?
            let text = DocumentExt::text(doc, i).expect("should have section");
            let title =
                DocumentExt::title(doc, i).expect("should have section");
            let embedding = model
                .encode(Into::<Cow<'a, str>>::into(text))
                .expect("should calculate embedding");
            let dist = embedding.cos(&word_embedding);
            ScoredLocation {
                score: dist,
                title: title.into(),
                location: Location {
                    uri: uri.clone(),
                    range: doc
                        .section_to_title_range(i)
                        .expect("Should have section"),
                },
            }
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{BertModel, Document};

    #[test]
    fn test_find_using_keyword() -> anyhow::Result<()> {
        let doc = Document::parse(
      "# Title\n\nThis is a document with a keyword.\n\n## Subtitle\n\nAnother keyword here.",
    )?;
        let model = BertModel::default();
        let uri = Url::parse("file:///home/user/document.md").unwrap();
        let keyword = "keyword";
        let results = find_by_keyword(uri, &model, &doc, keyword);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].title, "# Title");
        assert_eq!(results[1].title, "## Subtitle");

        Ok(())
    }
}
