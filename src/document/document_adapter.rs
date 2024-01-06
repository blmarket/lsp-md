use ropey::Rope;
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

impl LspAdapter for Rope {
    fn offset_to_position(
        &self,
        offset: usize,
    ) -> Option<tower_lsp::lsp_types::Position> {
        let slice = self.byte_slice(0..offset);
        let row = slice.len_lines() - 1;
        let col = slice.get_line(row)?.len_chars();
        Some(Position::new(row as u32, col as u32))
    }

    fn position_to_offset(
        &self,
        position: &tower_lsp::lsp_types::Position,
    ) -> Option<usize> {
        let line_offset = self.line_to_byte(position.line as usize);
        let char_offset = self.line(position.line as usize).slice(..position.character as usize).len_bytes();
        
        Some(line_offset + char_offset)
    }
}


#[cfg(test)]
mod tests {
    use ropey::Rope;
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
    
    #[test]
    fn test_offset_unicode() {
        use super::LspAdapter;
        
        let src = Rope::from_str("한글 텍스트\n 좋아요 좋아요\n");
        
        let pairs = vec![
            (0, Position::new(0, 0)),
            (6, Position::new(0, 2)),
            (7, Position::new(0, 3)),
            (10, Position::new(0, 4)),
            (16, Position::new(0, 6)),
            (17, Position::new(1, 0)),
            (18, Position::new(1, 1)),
        ];
        
        for (offset, pos) in pairs {
            assert_eq!(pos, src.offset_to_position(offset).unwrap());
            assert_eq!(offset, src.position_to_offset(&pos).unwrap());
        }
    }
}
