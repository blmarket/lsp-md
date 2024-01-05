use tower_lsp::lsp_types::{DidChangeTextDocumentParams, TextDocumentContentChangeEvent};

trait ApplyEdits: where Self: Sized {
    fn apply_edit(self, change: TextDocumentContentChangeEvent) -> Self;
    
    fn apply_edits(
        self,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Self {
        changes.into_iter().fold(self, |acc, edit| acc.apply_edit(edit))
    }
}

impl ApplyEdits for String {
    fn apply_edit(self, change: TextDocumentContentChangeEvent) -> Self {
        let Some(rng) = change.range else {
            return change.text;
        };
        return "".to_string();
    }
}
