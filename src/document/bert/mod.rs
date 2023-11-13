use std::sync::Mutex;

use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModel,
    SentenceEmbeddingsModelType,
};

use super::embedding::Embedding;

pub trait Encoder {
    fn encode_batch(
        &self,
        sentences: &[&str],
    ) -> anyhow::Result<Vec<Embedding>>;

    fn encode(&self, sentences: &str) -> anyhow::Result<Embedding> {
        self.encode_batch(&[sentences])
            .map(|v| v.into_iter().next().unwrap())
    }
}

pub struct Model {
    model: SentenceEmbeddingsModel,
}

impl Model {
    const MODEL_TYPE: SentenceEmbeddingsModelType =
        SentenceEmbeddingsModelType::AllMiniLmL12V2;

    pub fn load() -> anyhow::Result<Self> {
        let config = SentenceEmbeddingsConfig::from(Self::MODEL_TYPE);
        let model = SentenceEmbeddingsModel::new(config)?;
        Ok(Self { model })
    }
}

impl Encoder for Model {
    fn encode_batch(
        &self,
        sentences: &[&str],
    ) -> anyhow::Result<Vec<Embedding>> {
        let v1 = self.model.encode(sentences)?;

        let v2 =
            v1.into_iter().map(|v| {
                Embedding::new(v.try_into().unwrap_or_else(|_| {
                    panic!("Expected array with 384 elements")
                }))
            });

        Ok(v2.collect())
    }
}

impl<T> Encoder for Mutex<T>
where
    T: Encoder,
{
    fn encode_batch(
        &self,
        sentences: &[&str],
    ) -> anyhow::Result<Vec<Embedding>> {
        self.lock().unwrap().encode_batch(sentences)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_model_encode() -> anyhow::Result<()> {
        let model = Model::load()?;
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
        let model = Model::load()?;
        let sentence = "This is a test sentence.";
        let _ = model.encode(sentence)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_load_model_within_tokio() -> anyhow::Result<()> {
        let model =
            tokio::task::spawn_blocking(|| Model::load().unwrap()).await?;
        let embedding = model.encode("This is a test sentence.")?;
        let zero = Embedding::new(vec![1f32; 384]);
        assert!(zero.dist(&embedding) > 0.1);
        Ok(())
    }
}
