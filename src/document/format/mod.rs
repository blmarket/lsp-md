#[cfg(test)]
mod tests;

use std::sync::OnceLock;

use regex::Regex;
use tower_lsp::lsp_types::{Range, TextEdit, Position};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;

fn paragraph_separator() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF.get_or_init(|| {
        Regex::new(r#"(  \n+)|(\n\n+)"#).unwrap()
    })
}

fn whitespace() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF.get_or_init(|| {
        Regex::new(r#"[ \n]+"#).unwrap()
    })
}

pub trait LspRangeFormat {
    fn format(&self, range: Range) -> Option<Vec<TextEdit>>;
}

fn process_section(v: &mut Vec<TextEdit>, section: &str, offset: usize) {
    let mut col = 0i32;
    let mut space = -2i32;
    let mut space2 = -2i32;
    
    let ws = whitespace();
    let words = ws.find_iter(section);
    let mut word_offset = 0;
    for m in words {
        // FIXME: count unicode width of string here.
        // TODO: Place word, add line break if it exceeds 80 characters.
    }
}
    
impl<T> LspRangeFormat for T
where
    T: LspAdapter + SliceAccess,
{
    fn format(&self, range: Range) -> Option<Vec<TextEdit>> {
        // At this point we only support range formatting starting from a beginning of the line.
        let range_start_pos = Position {
            line: range.start.line,
            character: 0,
        };
        let Some(offset_start) = self.position_to_offset(&range_start_pos) else {
            return None;
        };
        let Some(offset_end) = self.position_to_offset(&range.end) else {
            return None;
        };

        let mut ret: Vec<TextEdit> = vec![];
        let slice = self.slice(offset_start..offset_end);
        
        let mut section_offset = 0;
        let sections = paragraph_separator().find_iter(&slice);
        for m in sections {
            let section = &slice[section_offset..m.start()];
            process_section(&mut ret, section, offset_start + section_offset);
            section_offset = m.end();
        }
        
        // for (pos, it) in slice.chars().enumerate() {
        //     match it {
        //         '\n' => {
        //             space = -2;
        //             space2 = -2;
        //             col = 0;
        //         },
        //         ' ' => {
        //             if space != col - 1 {
        //                 space2 = col;
        //             }
        //             space = col;
        //             col += 1;
        //         },
        //         _ => {
        //             col += 1;
        //             if col > 80 && space != -2 {
        //                 let start = self
        //                     .offset_to_position(
        //                         usize::try_from(
        //                             space2 +
        //                                 (offset_start as i32) +
        //                                 (pos as i32) -
        //                                 col +
        //                                 1,
        //                         )
        //                         .unwrap(),
        //                     )
        //                     .unwrap();
        //                 let end = self
        //                     .offset_to_position(
        //                         usize::try_from(
        //                             space +
        //                                 (offset_start as i32) +
        //                                 (pos as i32) -
        //                                 col +
        //                                 2,
        //                         )
        //                         .unwrap(),
        //                     )
        //                     .unwrap();
        //                 ret.push(TextEdit {
        //                     range: Range { start, end },
        //                     new_text: "\n".to_string(),
        //                 });
        //                 col -= space + 1;
        //                 space = -2;
        //                 space2 = -2;
        //             }
        //         },
        //     }
        //     // dbg!(format!("{} {} {}", it, col, space));
        // }

        Some(ret)
    }
}
