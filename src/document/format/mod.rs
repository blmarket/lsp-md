mod format_naive;
mod format_treesitter;
mod processors;
#[cfg(test)]
mod tests;
mod treesitter;
mod util;

use processors::{process_list_items, process_section};
use tower_lsp::lsp_types::{Range, TextEdit};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;

pub trait LspRangeFormat {
    fn format(&self, range: Range) -> Option<Vec<TextEdit>>;
}
