use rust_bert::pipelines::{
    keywords_extraction::{KeywordExtractionConfig, KeywordExtractionModel},
    sentence_embeddings::{
        SentenceEmbeddingsConfig, SentenceEmbeddingsModelType,
    },
};

use super::{
    embedding::Embedding,
    keywords::{Keyword, Keywords},
    Encoder,
};

pub struct BertModels<'a> {
    model: KeywordExtractionModel<'a>,
}

impl<'a> Default for BertModels<'a> {
    fn default() -> Self {
        let model = KeywordExtractionModel::new(KeywordExtractionConfig {
            sentence_embeddings_config: SentenceEmbeddingsConfig::from(
                SentenceEmbeddingsModelType::AllMiniLmL12V2,
            ),
            ..KeywordExtractionConfig::default()
        })
        .expect("Failed to load model");
        Self { model }
    }
}

impl<'a> Encoder for BertModels<'a> {
    fn encode_batch(
        &self,
        sentences: &[&str],
    ) -> anyhow::Result<Vec<Embedding>> {
        let v1 = self.model.sentence_embeddings_model.encode(sentences)?;

        let v2 =
            v1.into_iter().map(|v| {
                Embedding::new(v.try_into().unwrap_or_else(|_| {
                    panic!("Expected array with 384 elements")
                }))
            });

        Ok(v2.collect())
    }
}

impl<'a> Keywords for BertModels<'a> {
    fn extract_batch(
        &self,
        texts: &[&str],
    ) -> anyhow::Result<Vec<Vec<Keyword>>> {
        Ok(self
            .model
            .predict(texts)
            .unwrap()
            .into_iter()
            .map(|v2| v2.into_iter().map(|v3| v3.into()).collect())
            .collect())
    }
}
