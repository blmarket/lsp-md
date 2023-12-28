use std::sync::OnceLock;

use regex::Regex;

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

pub fn process_list_items(section: &str) -> String {
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

pub fn process_section(section: &str) -> String {
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
