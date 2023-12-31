use std::borrow::Cow;
use std::ops::{Bound, RangeBounds};

use tower_lsp::lsp_types::{Position, Range, TextEdit};

use super::{LspAdapter, SliceAccess};

pub struct TestDoc<'a>(pub &'a str);

impl TestDoc<'_> {
    #[allow(dead_code)] // Only used in tests.
    pub fn apply_edits(&self, edits: &[TextEdit]) -> String {
        let mut ret = String::with_capacity(self.0.len());
        let mut last = 0;
        for edit in edits {
            ret.push_str(
                &self.0
                    [last..self.position_to_offset(&edit.range.start).unwrap()],
            );
            ret.push_str(&edit.new_text);
            last = self.position_to_offset(&edit.range.end).unwrap();
        }
        ret.push_str(&self.0[last..]);
        ret
    }
}

impl<'a> SliceAccess for TestDoc<'a> {
    fn slice<'b, R: RangeBounds<usize>>(&'b self, r: R) -> Cow<'b, str> {
        // I'm dumb, I don't understand why I should do this.
        match (r.start_bound(), r.end_bound()) {
            (Bound::Unbounded, Bound::Unbounded) => Cow::Borrowed(self.0),
            (Bound::Unbounded, Bound::Included(&e)) => {
                Cow::Borrowed(&self.0[..=e])
            },
            (Bound::Unbounded, Bound::Excluded(&e)) => {
                Cow::Borrowed(&self.0[..e])
            },
            (Bound::Included(&s), Bound::Unbounded) => {
                Cow::Borrowed(&self.0[s..])
            },
            (Bound::Excluded(&s), Bound::Unbounded) => {
                Cow::Borrowed(&self.0[s + 1..])
            },
            (Bound::Included(&s), Bound::Included(&e)) => {
                Cow::Borrowed(&self.0[s..=e])
            },
            (Bound::Included(&s), Bound::Excluded(&e)) => {
                Cow::Borrowed(&self.0[s..e])
            },
            (Bound::Excluded(&s), Bound::Included(&e)) => {
                Cow::Borrowed(&self.0[s + 1..=e])
            },
            (Bound::Excluded(&s), Bound::Excluded(&e)) => {
                Cow::Borrowed(&self.0[s + 1..e])
            },
        }
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
