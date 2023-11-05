use std::ops::Range;

use regex::RegexBuilder;

#[derive(Debug, PartialEq)]
pub struct Sections {
    sections: Vec<Range<usize>>,
}

impl Sections {
    pub fn new(sections: Vec<Range<usize>>) -> Self {
        Sections { sections }
    }

    pub fn parse(src: &str) -> anyhow::Result<Self> {
        let re = RegexBuilder::new(r"^##? (.*)$").multi_line(true).build()?;
        let sections = re.find_iter(src).map(|it| it.range()).collect();

        Ok(Sections::new(sections))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!(
          Sections::new(vec![1..12, 32..44]),
          Sections::parse("\n# Section 1\n\nContents...\n\n---\n\n## Section 2\n\nContent of section 2...\n\n### Subsection").unwrap());
    }
}
