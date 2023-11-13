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

pub struct BertModels {
    model: KeywordExtractionModel<'static>,
}

impl Default for BertModels {
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

impl Encoder for BertModels {
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

impl Keywords for BertModels {
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_local_model_encode() -> anyhow::Result<()> {
        let model = BertModels::default();
        let sentences =
            vec!["This is a test sentence.", "This is another test sentence."];
        let embeddings = model.encode_batch(&sentences)?;
        assert_eq!(embeddings.len(), 2);

        let embedding = model.encode(&sentences[0])?;
        assert!(embeddings[0].dist(&embedding) < 0.001f32);

        Ok(())
    }

    #[test]
    fn test_local_model_encode_single() -> anyhow::Result<()> {
        let model = BertModels::default();
        let sentence = "This is a test sentence.";
        let _ = model.encode(sentence)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_load_model_within_tokio() -> anyhow::Result<()> {
        let model =
            tokio::task::spawn_blocking(|| BertModels::default()).await?;
        let embedding = model.encode("This is a test sentence.")?;
        let zero = Embedding::new(vec![1f32; 384]);
        assert!(zero.dist(&embedding) > 0.1);
        Ok(())
    }
}