use std::ops::Range;

use regex::RegexBuilder;

#[derive(Debug, PartialEq)]
pub struct Sections {
    sections: Vec<Range<usize>>,
}

impl Sections {
    pub fn new(tmp: Vec<Range<usize>>) -> Sections {
        Sections { sections: tmp }
    }
}

#[allow(dead_code)]
pub fn parse(src: &str) -> anyhow::Result<Sections> {
    let re = RegexBuilder::new(r"^##? (.*)$").multi_line(true).build()?;
    let tmp = re.find_iter(src).map(|it| it.range()).collect();

    Ok(Sections::new(tmp))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        assert_eq!( Sections::new(vec![1..13, 33..45]), parse(
            "\n## Section 1\n\nContents...\n\n---\n\n## Section 2\n\nContent of section 2...\n\n### Subsection").unwrap());
    }
}
