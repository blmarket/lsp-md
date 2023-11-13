use std::fs;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModelType,
};
use tower_lsp::lsp_types::Position;

use super::bert::Model;
use super::similar_notes::find_similar;
use super::Document;

struct TestSubject {
    pub model: Model,
    pub document: Document,
}

fn prepare_subject() -> anyhow::Result<TestSubject> {
    let model = Model::load()?;

    let contents = fs::read_to_string("test.md")
        .expect("Something went wrong reading the file");
    let document = Document::parse(contents)?;

    Ok(TestSubject { model, document })
}

#[test]
fn test_find_similar() -> anyhow::Result<()> {
    let TestSubject { model, document } = prepare_subject()?;

    let tmp = find_similar(&document, &model, Position::new(2, 0));

    dbg!(tmp);

    Ok(())
}

#[test]
fn test_bert() -> anyhow::Result<()> {
    let model = SentenceEmbeddingsBuilder::remote(
        SentenceEmbeddingsModelType::AllMiniLmL12V2,
    )
    .with_device(tch::Device::cuda_if_available())
    .create_model()?;

    // Define input
    let sentences =
        ["this is an example sentence", "each sentence is converted"];

    // Generate Embeddings
    let embeddings = model.encode(&sentences)?;
    assert_eq!(384, embeddings[0].len());

    Ok(())
}
