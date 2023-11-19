use std::sync::Mutex;

use super::embedding::Embedding;

pub trait Encoder {
    fn encode_batch<S: AsRef<str> + Sync>(
        &self,
        sentences: &[S],
    ) -> anyhow::Result<Vec<Embedding>>;

    fn encode<S: AsRef<str> + Sync>(
        &self,
        sentences: S,
    ) -> anyhow::Result<Embedding> {
        self.encode_batch(&[sentences])
            .map(|v| v.into_iter().next().unwrap())
    }
}

impl<T> Encoder for Mutex<T>
where
    T: Encoder,
{
    fn encode_batch<S: AsRef<str> + Sync>(
        &self,
        sentences: &[S],
    ) -> anyhow::Result<Vec<Embedding>> {
        self.lock().unwrap().encode_batch(sentences)
    }
}
