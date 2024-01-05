use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, TextEdit};

pub trait ApplyEdits: where Self: Sized {
    fn apply_change(self, change: TextDocumentContentChangeEvent) -> Self;
    
    fn apply_changes(
        self,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) -> Self {
        changes.into_iter().fold(self, |acc, edit| acc.apply_change(edit))
    }
    
    fn apply_edits(self, edits: impl AsRef<[TextEdit]>) -> Self {
        edits.as_ref().into_iter().fold(self, |acc, edit| acc.apply_change(TextDocumentContentChangeEvent {
            range: Some(edit.range),
            range_length: None,
            text: edit.new_text.clone(),
        }))
    }
}
