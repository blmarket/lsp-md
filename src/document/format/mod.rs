mod format_treesitter;
#[cfg(test)]
mod tests;
mod treesitter;
mod util;

use std::sync::OnceLock;

use regex::Regex;
use tower_lsp::lsp_types::{Position, Range, TextEdit};

use super::document::SliceAccess;
use super::document_adapter::LspAdapter;

fn paragraph_separator() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF.get_or_init(|| Regex::new(r#"(\n\n+)"#).unwrap());
}

fn non_ws() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF.get_or_init(|| Regex::new(r#"(  $)|([^ \n]+)"#).unwrap());
}

fn url() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF.get_or_init(|| Regex::new(r#"(https?://[^\s]+)"#).unwrap());
}

fn list_item() -> &'static Regex {
    static REF: OnceLock<Regex> = OnceLock::new();
    return REF
        .get_or_init(|| Regex::new(r#"(^\s*[-*] )|(\n\s*[-*] )"#).unwrap());
}

pub trait LspRangeFormat {
    fn format(&self, range: Range) -> Option<Vec<TextEdit>>;
}

fn process_list_items(section: &str) -> String {
    let words = non_ws();

    let mut ret = String::with_capacity(8192);
    let mut line_len: usize = 0;
    let mut line_indent: usize = 0;
    let mut was_url = false;

    for line in section.lines() {
        let m1 = list_item().find(line);
        let rest: &str = match m1 {
            Some(ms) => {
                if !ret.is_empty() {
                    ret.pop();
                    ret.push('\n');
                }
                ret.push_str(ms.as_str());
                line_len = ms.as_str().len();
                line_indent = line_len;
                line[ms.end()..].trim_start()
            },
            None => line.trim_start(),
        };

        let words = words.find_iter(rest);
        for m in words {
            let is_url = url().is_match(m.as_str());
            if line_len != line_indent &&
                (line_len + m.as_str().len() > 80 || is_url || was_url)
            {
                ret.pop();
                ret.push('\n');
                ret.push_str(&" ".repeat(line_indent));
                line_len = line_indent;
            }
            ret.push_str(m.as_str());
            ret.push(' ');
            line_len += m.as_str().len() + 1;
            was_url = is_url;
            if m.as_str() == "  " {
                ret.pop();
                was_url = true;
            }
        }
    }
    if ret.len() > 0 {
        ret.pop();
    }

    ret
}

fn process_section(section: &str) -> String {
    if list_item().is_match(section) {
        return process_list_items(section);
    }

    let ws = non_ws();
    let words = ws.find_iter(section);
    let mut ret = String::with_capacity(8192);
    let mut line_len = 0;
    let mut was_url = false;
    for m in words {
        let is_url = url().is_match(m.as_str());
        if line_len != 0 &&
            (line_len + m.as_str().len() > 80 || is_url || was_url)
        {
            ret.pop();
            ret.push('\n');
            line_len = 0;
        }
        ret.push_str(m.as_str());
        ret.push(' ');
        line_len += m.as_str().len() + 1;
        was_url = is_url;
    }
    if ret.len() > 0 {
        ret.pop();
    }
    ret
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
        let Some(offset_start) = self.position_to_offset(&range_start_pos)
        else {
            return None;
        };
        let Some(offset_end) = self.position_to_offset(&range.end) else {
            return None;
        };

        let mut updated = String::with_capacity(8192);
        let slice = self.slice(offset_start..offset_end);

        let mut section_offset = 0;
        let sections = paragraph_separator().find_iter(&slice);
        for m in sections {
            let section = &slice[section_offset..m.start()];
            updated.push_str(&process_section(section).as_str());
            updated.push_str(m.as_str());
            section_offset = m.end();
        }
        updated.push_str(&process_section(&slice[section_offset..]).as_str());

        Some(vec![TextEdit {
            range,
            new_text: updated,
        }])
    }
}
