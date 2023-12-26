use std::borrow::Cow;

use tower_lsp::lsp_types::Position;

use super::bert::{Keyword, Keywords};
use super::document::DocumentExt;
use super::document_adapter::DocumentLsp;

pub fn extract_keywords<'a, D>(
    doc: &'a D,
    enc: &impl Keywords,
    pos: &Position,
) -> anyhow::Result<Vec<Keyword>>
where
    D: DocumentLsp + DocumentExt<'a>,
{
    let current_section_idx =
        doc.position_to_section(pos).expect("Cannot find section");
    let text: Cow<'a, str> =
        DocumentExt::text(doc, current_section_idx)?.into();
    let keywords = enc.extract(&text)?;

    Ok(keywords)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::bert::BertModel;
    use crate::document::Document;

    const BUF: &'static str = r#"
# Section 1

Contents...

---

## Section 2

Content of section 2...

### Subsection"#;

    #[test]
    fn test_extract_keywords() -> anyhow::Result<()> {
        let doc = Document::parse(BUF)?;
        let pos = Position::new(1, 0);
        let enc = BertModel::default();

        let res = extract_keywords(&doc, &enc, &pos)?;

        dbg!(res);
        dbg!(extract_keywords(&doc, &enc, &Position::new(7, 0))?);

        Ok(())
    }
}
