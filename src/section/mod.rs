mod embedding;

use std::ops::Range;

use regex::RegexBuilder;

#[derive(Debug, PartialEq)]
pub struct Sections {
    document: String,
    sections: Vec<Range<usize>>,
}

impl Sections {
    fn new(sections: Vec<Range<usize>>, document: String) -> Self {
        Self { sections, document }
    }

    pub fn parse(src: &str) -> anyhow::Result<Self> {
        let re = RegexBuilder::new(r"^##? (.*)$").multi_line(true).build()?;
        let sections = re.find_iter(src).map(|it| it.range()).collect();

        Ok(Sections::new(sections, src.to_string()))
    }

    pub fn sections(&self) -> &Vec<Range<usize>> {
        return &self.sections;
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
    fn test() {
        assert_eq!(
            Sections::new(vec![1..12, 32..44], BUF.to_string()),
            Sections::parse(BUF).unwrap()
        );
    }
}
