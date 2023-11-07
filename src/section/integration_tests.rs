use rust_bert::{
    bert::BertEmbedding,
    pipelines::sentence_embeddings::SentenceEmbeddingsBuilder,
};
use std::fs;

use super::Sections;

#[test]
fn test() -> anyhow::Result<()> {
    let contents = fs::read_to_string("test.md")
        .expect("Something went wrong reading the file");
    let sections = Sections::parse(&contents)?;

    dbg!(&sections.document[sections.sections[0].0.clone()]);
    dbg!(&sections.document[sections.sections[1].0.clone()]);
    dbg!(&sections.document[sections.sections[2].0.clone()]);

    Ok(())
}

#[test]
fn test_bert() -> anyhow::Result<()> {
    let model = SentenceEmbeddingsBuilder::local("resources/")
        .with_device(tch::Device::cuda_if_available())
        .create_model()?;

    // Define input
    let sentences =
        ["this is an example sentence", "each sentence is converted"];

    // Generate Embeddings
    let embeddings = model.encode(&sentences)?;
    println!("{embeddings:?}");

    Ok(())
}
