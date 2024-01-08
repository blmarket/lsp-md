use ropey::Rope;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, TextEdit};

use super::document_adapter::LspAdapter;

pub trait IncrementalSync
where
    Self: Sized,
{
    fn apply_change(self, change: TextDocumentContentChangeEvent) -> Self;

    fn apply_changes(
        self,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Self {
        changes
            .into_iter()
            .fold(self, |acc, edit| acc.apply_change(edit))
    }

    fn apply_edits(self, edits: impl AsRef<[TextEdit]>) -> Self {
        edits.as_ref().into_iter().fold(self, |acc, edit| {
            acc.apply_change(TextDocumentContentChangeEvent {
                range: Some(edit.range),
                range_length: None,
                text: edit.new_text.clone(),
            })
        })
    }
}

impl IncrementalSync for Rope {
    fn apply_change(mut self, change: TextDocumentContentChangeEvent) -> Self {
        let Some(rng) = change.range else {
            return Rope::from_str(&change.text);
        };
        let sp =
            self.byte_to_char(self.position_to_offset(&rng.start).unwrap());
        let ep = self.byte_to_char(self.position_to_offset(&rng.end).unwrap());
        let rhs = Rope::split_off(&mut self, ep);
        let _ = Rope::split_off(&mut self, sp);
        self.append(Rope::from_str(&change.text));
        self.append(rhs);
        self
    }
}

#[cfg(test)]
mod tests {
    use ropey::Rope;

    use super::IncrementalSync as _;

    #[test]
    fn apply_change_for_rope() {
        let mut src = Rope::from_str("안녕 세상아\n착하게 live\n");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 0,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 2,
                    },
                }),
                range_length: None,
                text: "Hello".to_string(),
            },
        );

        assert_eq!("Hello 세상아\n착하게 live\n", String::from(src));
    }

    #[test]
    fn apply_multiline_change_for_rope() {
        let mut src = Rope::from_str("안녕 세상아\n착하게 live\n");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 3,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: 1,
                        character: 1,
                    },
                }),
                range_length: None,
                text: "world - 엄".to_string(),
            },
        );

        assert_eq!("안녕 world - 엄하게 live\n", String::from(src));
    }

    #[test]
    fn insert_beginning_for_rope() {
        let mut src = Rope::from_str("pay\nload");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 0,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 0,
                    },
                }),
                range_length: None,
                text: "insert ".to_string(),
            },
        );

        assert_eq!("insert pay\nload", String::from(src));
    }

    #[test]
    fn append_for_rope() {
        let mut src = Rope::from_str("pay\nload");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: 1,
                        character: 4,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: 1,
                        character: 4,
                    },
                }),
                range_length: None,
                text: "insert".to_string(),
            },
        );

        assert_eq!("pay\nloadinsert", String::from(src));
    }

    #[test]
    fn replace_whole_for_rope() {
        let mut src = Rope::from_str("pay\nload");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: Some(tower_lsp::lsp_types::Range {
                    start: tower_lsp::lsp_types::Position {
                        line: 0,
                        character: 0,
                    },
                    end: tower_lsp::lsp_types::Position {
                        line: 1,
                        character: 4,
                    },
                }),
                range_length: None,
                text: "insert".to_string(),
            },
        );

        assert_eq!("insert", String::from(src));
    }

    #[test]
    fn fullsync_for_rope() {
        let mut src = Rope::from_str("pay\nload");
        src = src.apply_change(
            tower_lsp::lsp_types::TextDocumentContentChangeEvent {
                range: None,
                range_length: None,
                text: "insert".to_string(),
            },
        );

        assert_eq!("insert", String::from(src));
    }
}
