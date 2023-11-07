mod embedding;
mod sqlite;
#[cfg(test)]
mod integration_tests;

use std::ops::Range;

use regex::RegexBuilder;

#[derive(Debug, PartialEq)]
pub struct Sections {
    document: String,
    sections: Vec<(Range<usize>, Range<usize>)>,
}

impl Sections {
    fn new(sections: Vec<(Range<usize>, Range<usize>)>, document: String) -> Self {
        Self { sections, document }
    }

    pub fn parse(src: &str) -> anyhow::Result<Self> {
        let re = RegexBuilder::new(r"^##? (.*)$").multi_line(true).build()?;
        let mut sections: Vec<(Range<usize>, Range<usize>)> = Vec::new();
        let mut last: usize = 0;
        let mut prev_title: Range<usize> = 0..0;
        for it in re.find_iter(src) {
            if last != 0 {
                sections.push((prev_title, last..it.start()));
            }
            prev_title = it.range();
            last = it.start();
        }
        
        if last != 0 {
            sections.push((prev_title, last..src.len()));
        }

        Ok(Sections::new(sections, src.to_string()))
    }

    pub fn sections(&self) -> Vec<Range<usize>> {
        return self.sections.clone().into_iter().map(|v| v.0).collect();
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
            Sections::new(vec![(1..12, 1..32), (32..44, 32..85)], BUF.to_string()),
            Sections::parse(BUF).unwrap()
        );
    }
}
