use serde::de::{self, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, PartialEq)]
pub struct Embedding([f32; 384]);

impl Embedding {
    // Create Embedding, with normalization
    pub fn new(v: Vec<f32>) -> Self {
        let mut sum: f32 = 0.0;
        for i in 0..384 {
            sum += v[i].powi(2);
        }
        if !sum.is_normal() {
            return Embedding([0.0; 384]);
        }
        let norm = sum.sqrt();

        let mut arr = [0.0; 384];
        for i in 0..384 {
            arr[i] = v[i] / norm;
        }

        Embedding(arr)
    }

    // Calculate cosine similarity. As the vectors are normalized, this is
    // equivalent to the dot product.
    pub fn cos(&self, other: &Self) -> f32 {
        let mut sum = 0.0;
        for i in 0..384 {
            sum += self.0[i] * other.0[i]
        }
        sum
    }
}

impl Serialize for Embedding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Embedding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmbeddingVisitor;

        impl<'de> Visitor<'de> for EmbeddingVisitor {
            type Value = Embedding;

            fn expecting(
                &self,
                formatter: &mut std::fmt::Formatter,
            ) -> std::fmt::Result {
                formatter.write_str("an array of 384 f32 values")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut arr = [0.0; 384];
                for i in 0..384 {
                    arr[i] = seq
                        .next_element()?
                        .ok_or_else(|| de::Error::invalid_length(i, &self))?;
                }
                Ok(Embedding(arr))
            }
        }

        deserializer.deserialize_seq(EmbeddingVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() -> anyhow::Result<()> {
        let value = Embedding([42.14; 384]);
        let mut buf = Vec::<u8>::new();
        let cursor = std::io::Cursor::new(&mut buf);
        ciborium::into_writer(&value, cursor).unwrap();
        assert_eq!(buf.len(), 1923);
        let decoded: Embedding =
            ciborium::from_reader(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(value, decoded);

        Ok(())
    }

    #[test]
    fn test_embedding_with_zero() -> anyhow::Result<()> {
        let value = Embedding([0.0; 384]);
        let mut buf = Vec::<u8>::new();
        let cursor = std::io::Cursor::new(&mut buf);
        ciborium::into_writer(&value, cursor).unwrap();
        assert_eq!(buf.len(), 1155);
        let decoded: Embedding =
            ciborium::from_reader(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(value, decoded);

        Ok(())
    }

    #[test]
    fn test_embedding_normalization() -> anyhow::Result<()> {
        let value = Embedding::new(vec![0.0; 384]);
        let mut buf = Vec::<u8>::new();
        let cursor = std::io::Cursor::new(&mut buf);
        ciborium::into_writer(&value, cursor).unwrap();
        assert_eq!(buf.len(), 1155);
        let decoded: Embedding =
            ciborium::from_reader(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(value, decoded);

        Ok(())
    }

    #[test]
    fn test_embedding_subnormal() -> anyhow::Result<()> {
        let mut emb = vec![0.0; 384];
        emb[0] = 1e-20;
        let value = Embedding::new(emb);
        assert_eq!(0.0, value.0[0]);

        Ok(())
    }
}
