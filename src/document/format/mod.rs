#[cfg(test)]
mod tests;

use tower_lsp::lsp_types::{Range, TextEdit};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;

pub trait LspRangeFormat {
    fn format(&self, range: Range) -> Option<Vec<TextEdit>>;
}

impl<T> LspRangeFormat for T
where
    T: LspAdapter + SliceAccess,
{
    fn format(&self, range: Range) -> Option<Vec<TextEdit>> {
        let Some(offset_start) = self.position_to_offset(&range.start) else {
            return None;
        };
        let Some(offset_end) = self.position_to_offset(&range.end) else {
            return None;
        };

        let mut col: i32 = range.start.character as i32;
        let mut space: i32 = -2;
        let mut space2: i32 = -2;
        let mut ret: Vec<TextEdit> = vec![];
        let slice = self.slice(offset_start..offset_end);
        for (pos, it) in slice.chars().enumerate() {
            match it {
                '\n' => {
                    space = -2;
                    space2 = -2;
                    col = 0;
                },
                ' ' => {
                    if space != col - 1 {
                        space2 = col;
                    }
                    space = col;
                    col += 1;
                },
                _ => {
                    col += 1;
                    if col > 80 && space != -2 {
                        let start = self
                            .offset_to_position(
                                usize::try_from(
                                    space2 +
                                        (offset_start as i32) +
                                        (pos as i32) -
                                        col +
                                        1,
                                )
                                .unwrap(),
                            )
                            .unwrap();
                        let end = self
                            .offset_to_position(
                                usize::try_from(
                                    space +
                                        (offset_start as i32) +
                                        (pos as i32) -
                                        col +
                                        2,
                                )
                                .unwrap(),
                            )
                            .unwrap();
                        ret.push(TextEdit {
                            range: Range { start, end },
                            new_text: "\n".to_string(),
                        });
                        col -= space + 1;
                        space = -2;
                        space2 = -2;
                    }
                },
            }
            // dbg!(format!("{} {} {}", it, col, space));
        }

        Some(ret)
    }
}
