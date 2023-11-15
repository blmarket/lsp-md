use tower_lsp::lsp_types::{Position, Range};

use super::document::BasicDocument;

pub trait DocumentAdapter: BasicDocument {
    fn offset_to_position(&self, offset: usize) -> Option<Position> {
        // FIXME: improve performance by indexing all newlines
        let lines = self.contents()[0..offset].split('\n');
        let acc = lines.fold((0, 0), |(line, _), v| (line + 1, v.len()));

        Some(Position::new(acc.0 - 1, acc.1 as u32))
    }

    fn position_to_offset(&self, position: &Position) -> Option<usize> {
        let lines = self.contents().lines().take((position.line) as usize);
        let acc = lines.fold(0, |acc, v| acc + v.len() + 1);

        Some(acc + position.character as usize)
    }

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
            tower_lsp::lsp_types::Range::new(start, end)
        })
    }

    fn section_titles(&self) -> Vec<tower_lsp::lsp_types::Range> {
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

        assert_eq!(9, doc.position_to_offset(&Position::new(1, 8)).unwrap());
        assert_eq!(32, doc.position_to_offset(&Position::new(7, 0)).unwrap());
    }
}
