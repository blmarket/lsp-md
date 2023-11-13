use std::sync::Mutex;

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
