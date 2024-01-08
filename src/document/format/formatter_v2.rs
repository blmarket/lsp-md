#![allow(dead_code)]

use ropey::Rope;
use tower_lsp::lsp_types::{
    Position as LspPosition, TextDocumentContentChangeEvent,
};
use tree_sitter::{InputEdit, Parser, Point, Tree};

use crate::document::document::SliceAccess;
use crate::document::document_adapter::LspAdapter;
use crate::document::incremental_sync::IncrementalSync;

pub struct Formatter {
    buf: Rope,
    tree: Tree,
}

impl Formatter {
    pub fn new(buf: Rope) -> Self {
        let lang = tree_sitter_md::language();
        let mut parser = Parser::new();
        parser.set_language(lang).expect("should set lang");

        let tree = parser
            .parse_with(
                &mut |offset, _| {
                    let (chunk_str, chunk_byte_idx, _, _) =
                        buf.chunk_at_byte(offset);
                    &chunk_str.as_bytes()[offset - chunk_byte_idx..]
                },
                None,
            )
            .expect("should parse doc");

        Self { buf, tree }
    }

    fn apply_edit(updated: Rope, mut old_tree: Tree, edit: InputEdit) -> Self {
        let lang = tree_sitter_md::language();
        let mut parser = Parser::new();
        parser.set_language(lang).expect("should set lang");

        old_tree.edit(&edit);
        let tree = parser
            .parse_with(
                &mut |offset, _| {
                    let (chunk_str, chunk_byte_idx, _, _) =
                        updated.chunk_at_byte(offset);
                    &chunk_str.as_bytes()[offset - chunk_byte_idx..]
                },
                Some(&old_tree),
            )
            .expect("should parse doc");

        Self { buf: updated, tree }
    }
}

trait LspPositionToPoint {
    fn to_point(&self) -> Point;
}

impl LspPositionToPoint for LspPosition {
    fn to_point(&self) -> Point {
        Point {
            row: self.line as usize,
            column: self.character as usize,
        }
    }
}

impl IncrementalSync for Formatter {
    fn apply_change(self, change: TextDocumentContentChangeEvent) -> Self {
        let Some(rng) = change.range else {
            // no range = full text change = creating a new Formatter
            return Formatter::new(Rope::from_str(&change.text));
        };

        let sp = self.buf.position_to_offset(&rng.start).unwrap();
        let old_ep = self.buf.position_to_offset(&rng.end).unwrap();
        let new_ep = sp + change.text.as_bytes().len();

        let updated_rope = self.buf.apply_change(change);

        let new_end_position = updated_rope.offset_to_position(new_ep).unwrap();

        let input_edit = InputEdit {
            start_byte: sp,
            old_end_byte: old_ep,
            new_end_byte: new_ep,
            start_position: rng.start.to_point(),
            old_end_position: rng.end.to_point(),
            new_end_position: new_end_position.to_point(),
        };

        Formatter::apply_edit(updated_rope, self.tree, input_edit)
    }
}

impl LspAdapter for Formatter {
    fn offset_to_position(&self, offset: usize) -> Option<LspPosition> {
        self.buf.offset_to_position(offset)
    }

    fn position_to_offset(&self, position: &LspPosition) -> Option<usize> {
        self.buf.position_to_offset(position)
    }
}

impl SliceAccess for Formatter {
    fn slice<
        'a,
        R: std::ops::RangeBounds<usize>
            + std::slice::SliceIndex<str, Output = str>,
    >(
        &'a self,
        r: R,
    ) -> std::borrow::Cow<'a, str> {
        self.buf.byte_slice(r).into()
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::*;
    use crate::document::format::treesitter::debug_walk;

    #[test]
    fn new_works() {
        let buf = Rope::from("Hello world\n한글\n".repeat(10000));
        let formatter = Formatter::new(buf);
        debug_walk(formatter.tree.walk());
    }

    #[test]
    fn partial_update_should_work() {
        let buf = Rope::from(
            "## Title\n\nSome paragraph\n\n```\nlet a = 1;\n```\n\n",
        );
        let formatter = Formatter::new(buf);
        debug_walk(formatter.tree.walk());

        let updated = formatter.apply_change(TextDocumentContentChangeEvent {
            range: Some(tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: 4,
                    character: 0,
                },
                end: tower_lsp::lsp_types::Position {
                    line: 5,
                    character: 0,
                },
            }),
            range_length: None,
            text: "".to_string(),
        });

        debug_walk(updated.tree.walk());
    }

    #[test]
    fn partial_update_should_handle_big_tree_changes() {
        let src =
            "## Title\n\n".to_string() + &"Some paragraph\n\n".repeat(10000);
        let buf = Rope::from_str(&src);
        let formatter = Formatter::new(buf);
        debug_walk(formatter.tree.walk());

        let updated = formatter.apply_change(TextDocumentContentChangeEvent {
            range: Some(tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: 4,
                    character: 0,
                },
                end: tower_lsp::lsp_types::Position {
                    line: 5,
                    character: 0,
                },
            }),
            range_length: None,
            text: "```\n".to_string(),
        });

        debug_walk(updated.tree.walk());

        let updated2 = updated.apply_change(TextDocumentContentChangeEvent {
            range: Some(tower_lsp::lsp_types::Range {
                start: tower_lsp::lsp_types::Position {
                    line: 20002,
                    character: 0,
                },
                end: tower_lsp::lsp_types::Position {
                    line: 20002,
                    character: 0,
                },
            }),
            range_length: None,
            text: "```\n".to_string(),
        });

        debug_walk(updated2.tree.walk());
    }
}
