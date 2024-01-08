#![allow(dead_code)]

use std::borrow::Cow;
use std::ops::{Range, RangeBounds};
use std::slice::SliceIndex;

use regex::RegexBuilder;
use ropey::Rope;
use tower_lsp::lsp_types::Position;

use super::document::{BasicDocument, Section, SliceAccess};
use super::document_adapter::{DocumentLsp, LspAdapter};
use super::format::FormatterV2;

pub struct Document(FormatterV2, Vec<Section>);

impl SliceAccess for Document {
    fn slice<'a, R: RangeBounds<usize> + SliceIndex<str, Output = str>>(
        &'a self,
        r: R,
    ) -> Cow<'a, str> {
        self.0.slice(r)
    }
}

impl LspAdapter for Document {
    fn offset_to_position(&self, offset: usize) -> Option<Position> {
        self.0.offset_to_position(offset)
    }

    fn position_to_offset(&self, position: &Position) -> Option<usize> {
        self.0.position_to_offset(position)
    }
}

impl BasicDocument for Document {
    type Output = Vec<Section>;

    fn sections(&self) -> Self::Output {
        self.0.sections()
    }
}

impl DocumentLsp for Document {}

impl Document {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        Document::from_str(text)
    }

    pub fn from_str(text: &str) -> anyhow::Result<Self> {
        let rope = Rope::from_str(&text);
        let re = RegexBuilder::new(r"^##? (.*)$").multi_line(true).build()?;
        let mut sections: Vec<Section> = Vec::new();
        let mut prev_title: Range<usize> = 0..0;
        for it in re.find_iter(&text) {
            if prev_title.end != 0 {
                sections.push(Section {
                    title: prev_title.clone(),
                    range: prev_title.start..it.start(),
                });
            }
            prev_title = it.range();
        }

        if prev_title.end != 0 {
            sections.push(Section {
                title: prev_title.clone(),
                range: prev_title.start..text.len(),
            });
        }

        Ok(Self(FormatterV2::new(rope), sections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BUF: &'static str = r#"
# Section 1

Contents...

---

## Section 2

Content of section 2...

### Subsection"#;

    #[test]
    fn test_parse() {
        assert_eq!(
            vec![
                Section {
                    title: 1..12,
                    range: 1..32
                },
                Section {
                    title: 32..44,
                    range: 32..85
                }
            ],
            Document::from_str(BUF).unwrap().sections(),
        );
    }
}
