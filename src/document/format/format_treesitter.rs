use std::ops::Range;

use tree_sitter::Node;

use super::{LspAdapter, LspRangeFormat, SliceAccess, treesitter::Traversal};

struct Tmp<'a, T: LspAdapter + SliceAccess> {
    buf: T,
    tree_root: Node<'a>,
}

impl<'a, T: LspAdapter + SliceAccess> Tmp<'a, T> {
    fn range_from_lsp(
        &self,
        range: tower_lsp::lsp_types::Range,
    ) -> Range<usize> {
        self.buf.position_to_offset(&range.start).unwrap()..
            self.buf.position_to_offset(&range.end).unwrap()
    }
}

impl<'a, T: LspAdapter + SliceAccess> LspRangeFormat for Tmp<'a, T> {
    fn format(
        &self,
        range: tower_lsp::lsp_types::Range,
    ) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
        let r2 = self.range_from_lsp(range);
        let cursor = self.tree_root.walk();
        let trav = Traversal::from_cursor(cursor);
        for it in trav {
            let r1 = it.byte_range();
            if r1.start >= r2.end || r2.start >= r1.end {
                continue;
            }
            dbg!(it);
        }
        
        todo!()
    }
}
