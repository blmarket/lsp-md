use std::fs;
use std::str::FromStr;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use tower_lsp::lsp_types::{Position, Url};

use super::{BertModel, Document};
use crate::document::find_similar;

struct TestSubject {
    pub model: BertModel,
    pub document: Document,
}

fn prepare_subject() -> anyhow::Result<TestSubject> {
    let model = BertModel::default();

    let contents = fs::read_to_string("examples/test.md")
        .expect("Something went wrong reading the file");
    let document = Document::parse(contents)?;

    Ok(TestSubject { model, document })
}

#[test]
fn test_find_similar() -> anyhow::Result<()> {
    let TestSubject { model, document } = prepare_subject()?;

    let tmp = find_similar(
        Url::from_str("test://file")?,
        &document,
        &model,
        &Position::new(1, 0),
    );

    dbg!(tmp);

    Ok(())
}
