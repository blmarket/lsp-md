use std::ops::Range;

use regex::RegexBuilder;
use tower_lsp::lsp_types::Position;

#[derive(Debug, PartialEq)]
pub struct Document {
    text: String,
    sections: Vec<Section>,
}

#[derive(Debug, PartialEq)]
pub struct Section {
    title: Range<usize>,
    range: Range<usize>,
}

pub trait BasicDocument {
    fn contents(&self) -> &str;
    fn sections(&self) -> &[Section];
}

pub trait BasicDocumentExt: BasicDocument {
    fn title(&self, index: usize) -> anyhow::Result<&str> {
        self.sections()
            .get(index)
            .map(|v| &self.contents()[v.title.clone()])
            .ok_or(anyhow::anyhow!("index out of range"))
    }

    fn text(&self, index: usize) -> anyhow::Result<&str> {
        self.sections()
            .get(index)
            .map(|v| &self.contents()[v.range.clone()])
            .ok_or(anyhow::anyhow!("index out of range"))
    }
}

pub trait DocumentAdapter: BasicDocument {
    fn offset_to_position(&self, offset: usize) -> Option<Position> {
        // FIXME: improve performance by indexing all newlines
        let lines = self.contents()[0..offset].split('\n');
        let acc = lines.fold((0, 0), |(line, _), v| (line + 1, v.len()));

        Some(Position::new(acc.0 - 1, acc.1 as u32))
    }

    fn position_to_offset(&self, position: Position) -> Option<usize> {
        let lines = self.contents().lines().take((position.line) as usize);
        let acc = lines.fold(0, |acc, v| acc + v.len() + 1);

        Some(acc + position.character as usize)
    }

    fn position_to_section(&self, position: Position) -> Option<usize> {
        let offset = self.position_to_offset(position)?;
        self.sections()
            .iter()
            .enumerate()
            .find(|(_, v)| v.range.contains(&offset))
            .map(|(i, _)| i)
    }

    fn section_to_title_range(
        &self,
        index: usize,
    ) -> Option<tower_lsp::lsp_types::Range> {
        self.sections().get(index).map(|v| {
            let start = self.offset_to_position(v.title.start).unwrap();
            let end = self.offset_to_position(v.title.end).unwrap();
            tower_lsp::lsp_types::Range::new(start, end)
        })
    }
}

impl BasicDocument for Document {
    fn contents(&self) -> &str {
        &self.text
    }

    fn sections(&self) -> &[Section] {
        &self.sections
    }
}

impl BasicDocumentExt for Document {}

impl DocumentAdapter for Document {}

impl Document {
    pub fn parse(text: String) -> anyhow::Result<Self> {
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

        Ok(Document { sections, text })
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
            Document {
                text: BUF.to_string(),
                sections: vec![
                    Section {
                        title: 1..12,
                        range: 1..32
                    },
                    Section {
                        title: 32..44,
                        range: 32..85
                    }
                ]
            },
            Document::parse(BUF.to_string()).unwrap()
        );
    }

    #[test]
    fn test_offset_conversion() {
        use super::DocumentAdapter;

        let doc = Document::parse(BUF.to_string()).unwrap();
        assert_eq!(Position::new(0, 0), doc.offset_to_position(0).unwrap());
        assert_eq!(Position::new(1, 0), doc.offset_to_position(1).unwrap());
        assert_eq!(Position::new(1, 8), doc.offset_to_position(9).unwrap());
        assert_eq!(Position::new(1, 9), doc.offset_to_position(10).unwrap());
        assert_eq!(Position::new(1, 10), doc.offset_to_position(11).unwrap());
        assert_eq!(Position::new(1, 11), doc.offset_to_position(12).unwrap());
        assert_eq!(Position::new(7, 0), doc.offset_to_position(32).unwrap());
        assert_eq!(Position::new(7, 12), doc.offset_to_position(44).unwrap());

        assert_eq!(9, doc.position_to_offset(Position::new(1, 8)).unwrap());
        assert_eq!(32, doc.position_to_offset(Position::new(7, 0)).unwrap());
    }
}
