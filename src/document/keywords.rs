use rust_bert::pipelines::keywords_extraction::{
    Keyword, KeywordExtractionConfig, KeywordExtractionModel,
};
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsConfig, SentenceEmbeddingsModelType,
};

use super::bert::Encoder;
use super::embedding::Embedding;

struct BertModels<'a> {
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

pub trait Keywords {
    fn extract_batch(
        &self,
        texts: &[&str],
    ) -> anyhow::Result<Vec<Vec<Keyword>>>;

    fn extract(&self, text: &str) -> anyhow::Result<Vec<Keyword>> {
        self.extract_batch(&[text])
            .map(|v| v.into_iter().next().unwrap())
    }
}

impl<'a> Keywords for BertModels<'a> {
    fn extract_batch(
        &self,
        texts: &[&str],
    ) -> anyhow::Result<Vec<Vec<Keyword>>> {
        Ok(self.model.predict(texts).expect("Failed to run model"))
    }
}

#[cfg(test)]
mod tests {
    use rust_bert::pipelines::keywords_extraction::{
        KeywordExtractionConfig, KeywordExtractionModel,
    };
    use rust_bert::pipelines::sentence_embeddings::{
        SentenceEmbeddingsConfig, SentenceEmbeddingsModelType,
    };

    use super::*;

    const TEST_SECTION: &str = r#"
## Implement my own reference list viewer

NeoVim's own implementation of listing references:
https://github.com/neovim/neovim/blob/ae8ca79920a8d0e928ac1502a10d1d063a06cae5/runtime/lua/vim/lsp/handlers.lua#L231

Seems I don't want it to be sorted by locations. I want it to be sorted by
their similarity score.

So far telescope or fzf plugins seems to provide similar listing functionality,
but also I don't want them to sort the list by fuzzy finder algorithms.

I need to run :lua vim.lsp.codelens.refresh() in order to show codelens
actions. Not sure how to make it follow-up the document changes properly."#;

    #[test]
    fn test_basic_extraction() -> anyhow::Result<()> {
        let model = KeywordExtractionModel::new(KeywordExtractionConfig {
            sentence_embeddings_config: SentenceEmbeddingsConfig::from(
                SentenceEmbeddingsModelType::AllMiniLmL12V2,
            ),
            ..KeywordExtractionConfig::default()
        })?;
        let _ = model.predict(&[TEST_SECTION]);
        Ok(())
    }

    #[test]
    fn test_using_module() -> anyhow::Result<()> {
        let models = BertModels::default();
        let _ = models.extract(TEST_SECTION);
        Ok(())
    }
}
