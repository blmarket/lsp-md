pub struct Keyword {
    pub score: f32,
    pub text: String,
}

impl Into<Keyword> for rust_bert::pipelines::keywords_extraction::Keyword {
    fn into(self) -> Keyword {
        Keyword {
            score: self.score,
            text: self.text,
        }
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
        let models = super::super::model::BertModels::default();
        let _ = models.extract(TEST_SECTION);
        Ok(())
    }
}
