mod format_treesitter;
mod formatter_v2;
mod processors;
#[cfg(test)]
mod tests;
mod treesitter;

use processors::{process_list_items, process_section};
use tower_lsp::lsp_types::{Range, TextEdit};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;

pub trait LspRangeFormat {
    fn format(&self, range: Range) -> Option<Vec<TextEdit>>;
}
/// A formatter that uses the treesitter library to format documents.
pub use format_treesitter::Formatter;
pub use formatter_v2::Formatter as FormatterV2;
