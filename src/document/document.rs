use std::ops::Range;

use regex::RegexBuilder;
use ropey::Rope;

use super::document_adapter::DocumentAdapter;

#[derive(Debug, PartialEq)]
pub struct Document {
    rope: Rope,
    text: String,
    sections: Vec<Section>,
}

#[derive(Debug, PartialEq)]
pub struct Section {
    pub(super) title: Range<usize>,
    pub(super) range: Range<usize>,
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

        Ok(Document {
            text,
            rope,
            sections,
        })
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
            Document::parse(BUF.to_string()).unwrap().sections,
        );
    }
}
