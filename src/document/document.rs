use std::{borrow::Cow, ops::RangeBounds};
use std::ops::Range;

use regex::RegexBuilder;
use ropey::{Rope, RopeSlice};
use tower_lsp::lsp_types::Position;

use super::document_adapter::{DocumentLsp, LspAdapter};

#[derive(Debug, PartialEq)]
pub struct Document {
    rope: Rope,
    sections: Vec<Section>,
}

pub trait SliceAccess {
    fn slice<'a, R: RangeBounds<usize>>(&'a self, r: R) -> Cow<'a, str>;
}

impl SliceAccess for Document {
    fn slice<'a, R: RangeBounds<usize>>(&'a self, r: R) -> Cow<'a, str> {
        self.rope.get_byte_slice(r).map(|v| v.into()).unwrap()
    }
}

impl LspAdapter for Document {
    fn offset_to_position(
        &self,
        offset: usize,
    ) -> Option<tower_lsp::lsp_types::Position> {
        let slice = self.rope.byte_slice(0..offset);
        let row = slice.len_lines() - 1;
        let col = slice.get_line(row)?.len_chars();
        Some(Position::new(row as u32, col as u32))
    }

    fn position_to_offset(
        &self,
        position: &tower_lsp::lsp_types::Position,
    ) -> Option<usize> {
        // FIXME: It will be wrong if the line has multi-byte characters.
        self.rope
            .try_line_to_char(position.line as usize)
            .map(|v| v + position.character as usize)
            .ok()
    }
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub(super) title: Range<usize>,
    pub(super) range: Range<usize>,
}

pub trait BasicDocument {
    fn sections(&self) -> &[Section];
}

pub trait DocumentExt<'a> {
    fn title(&'a self, index: usize) -> anyhow::Result<RopeSlice<'a>>;
    fn text(&'a self, index: usize) -> anyhow::Result<RopeSlice<'a>>;
}

impl<'a> DocumentExt<'a> for Document {
    fn title(&'a self, index: usize) -> anyhow::Result<RopeSlice<'a>> {
        let section = self
            .sections()
            .get(index)
            .ok_or(anyhow::anyhow!("index out of range"))?;
        Ok(self.rope.byte_slice(section.title.clone()))
    }

    fn text(&'a self, index: usize) -> anyhow::Result<RopeSlice<'a>> {
        let section = self
            .sections()
            .get(index)
            .ok_or(anyhow::anyhow!("index out of range"))?;
        Ok(self.rope.byte_slice(section.range.clone()))
    }
}

impl BasicDocument for Document {
    fn sections(&self) -> &[Section] {
        &self.sections
    }
}

impl DocumentLsp for Document {}

impl Document {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
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

        Ok(Document { rope, sections })
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
            Document::parse(BUF).unwrap().sections,
        );
    }
}
