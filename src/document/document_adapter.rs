use tower_lsp::lsp_types::{Position, Range};

use super::document::BasicDocument;

pub trait LspAdapter {
    fn offset_to_position(&self, offset: usize) -> Option<Position>;
    fn position_to_offset(&self, position: &Position) -> Option<usize>;
}

pub trait DocumentLsp: BasicDocument + LspAdapter {
    fn position_to_section(&self, position: &Position) -> Option<usize> {
        let offset = self.position_to_offset(position)?;
        self.sections()
            .iter()
            .enumerate()
            .find(|(_, v)| v.range.contains(&offset))
            .map(|(i, _)| i)
    }

    fn section_to_title_range(&self, index: usize) -> Option<Range> {
        self.sections().get(index).map(|v| {
            let start = self.offset_to_position(v.title.start).unwrap();
            let end = self.offset_to_position(v.title.end).unwrap();
            Range::new(start, end)
        })
    }

    fn section_titles(&self) -> Vec<Range> {
        (0..self.sections().len())
            .map(|s| self.section_to_title_range(s).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use tower_lsp::lsp_types::Position;

    use crate::document::Document;

    const BUF: &'static str = r#"
# Section 1

Contents...

---

## Section 2

Content of section 2...

### Subsection"#;

    #[test]
    fn test_offset_conversion() {
        use super::LspAdapter;

        let doc = Document::parse(BUF).unwrap();
        assert_eq!(Position::new(0, 0), doc.offset_to_position(0).unwrap());
        assert_eq!(Position::new(1, 0), doc.offset_to_position(1).unwrap());
        assert_eq!(Position::new(1, 8), doc.offset_to_position(9).unwrap());
        assert_eq!(Position::new(1, 9), doc.offset_to_position(10).unwrap());
        assert_eq!(Position::new(1, 10), doc.offset_to_position(11).unwrap());
        assert_eq!(Position::new(1, 11), doc.offset_to_position(12).unwrap());
        assert_eq!(Position::new(7, 0), doc.offset_to_position(32).unwrap());
        assert_eq!(Position::new(7, 12), doc.offset_to_position(44).unwrap());

        assert_eq!(9, doc.position_to_offset(&Position::new(1, 8)).unwrap());
        assert_eq!(32, doc.position_to_offset(&Position::new(7, 0)).unwrap());
    }
}
