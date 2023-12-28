use std::borrow::Cow;

use tower_lsp::lsp_types::{TextEdit, Range, Position};
use super::{LspAdapter, SliceAccess};

pub struct TestDoc<'a>(pub &'a str);

impl TestDoc<'_> {
    #[allow(dead_code)] // Only used in tests.
    pub fn apply_edits(&self, edits: &[TextEdit]) -> String {
        let mut ret = String::with_capacity(self.0.len());
        let mut last = 0;
        for edit in edits {
            ret.push_str(&self.0[last..self.position_to_offset(&edit.range.start).unwrap()]);
            ret.push_str(&edit.new_text);
            last = self.position_to_offset(&edit.range.end).unwrap();
        }
        ret.push_str(&self.0[last..]);
        ret
    }
}

impl<'a> SliceAccess for TestDoc<'a> {
    fn slice<'b>(&'b self, r: std::ops::Range<usize>) -> Cow<'b, str> {
        Cow::Borrowed(&self.0[r])
    }
}

impl<'a> LspAdapter for TestDoc<'a> {
    fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.0.len() {
            return None;
        }

        let lines = self.0[0..offset].split("\n");

        let (line, last) =
            lines.fold((0u32, None), |(line, _), it| (line + 1, Some(it)));

        let Some(tmp) = last else { return None };

        Some(Position {
            line: line - 1,
            character: tmp.chars().count() as u32,
        })
    }

    fn position_to_offset(&self, position: &Position) -> Option<usize> {
        let mut ret = 0;
        for (a, b) in self.0.split("\n").enumerate() {
            if a == position.line as usize {
                if position.character == 0 {
                    return Some(ret);
                }
                let Some((pos, chr)) =
                    b.char_indices().nth((position.character - 1) as usize)
                else {
                    return None;
                };
                return Some(ret + pos + chr.len_utf8());
            }
            ret += b.len() + 1; // 1 for '\n'.
        }
        None
    }
}

pub trait QuickEdit {
    fn edit<S: ToString>(
        &self,
        soff: usize,
        eoff: usize,
        new_text: S,
    ) -> TextEdit;
}

impl<T> QuickEdit for T
where
    T: LspAdapter,
{
    fn edit<S: ToString>(
        &self,
        soff: usize,
        eoff: usize,
        new_text: S,
    ) -> TextEdit {
        let start = self.offset_to_position(soff).unwrap();
        let end = self.offset_to_position(eoff).unwrap();
        TextEdit {
            range: Range { start, end },
            new_text: new_text.to_string(),
        }
    }
}
