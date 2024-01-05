use std::borrow::Cow;
use std::ops::RangeBounds;
use std::slice::SliceIndex;

use tower_lsp::lsp_types::{Position, TextDocumentContentChangeEvent};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;
use super::incremental_sync::ApplyEdits;

#[derive(Debug, PartialEq)]
pub struct TestDoc2(String);

impl TestDoc2 {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

impl Into<String> for TestDoc2 {
    fn into(self) -> String {
        self.0
    }
}

impl PartialEq<TestDoc2> for String {
    fn eq(&self, other: &TestDoc2) -> bool {
        self == &other.0
    }
}

impl PartialEq<TestDoc2> for &str {
    fn eq(&self, other: &TestDoc2) -> bool {
        self == &other.0
    }
}

impl ApplyEdits for TestDoc2 {
    fn apply_change(self, change: TextDocumentContentChangeEvent) -> Self {
        let Some(rng) = change.range else {
            return TestDoc2::new(change.text);
        };
        let mut ret = String::with_capacity(self.0.len() + change.text.len());
        let sp = self.position_to_offset(&rng.start).unwrap();
        let ep = self.position_to_offset(&rng.end).unwrap();
        ret.push_str(&self.0[..sp]);
        ret.push_str(&change.text);
        ret.push_str(&self.0[ep..]);

        TestDoc2::new(ret)
    }
}

impl SliceAccess for TestDoc2 {
    fn slice<'b, R>(&'b self, r: R) -> Cow<'b, str> 
    where R: RangeBounds<usize> + SliceIndex<str, Output = str>,
    {
        Cow::Borrowed(&self.0.as_str()[r])
    }
}

impl LspAdapter for TestDoc2 {
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