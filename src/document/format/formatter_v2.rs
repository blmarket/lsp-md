#![allow(dead_code)]

use ropey::Rope;
use tree_sitter::{Parser, Tree};

pub struct Formatter<'a> {
    buf: &'a Rope,
    tree: Tree,
}

impl<'a> Formatter<'a> {
    pub fn new(buf: &'a Rope) -> Self {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_format() {
        let buf = Rope::from("Hello world\n한글\n".repeat(10000));
        let formatter = Formatter::new(&buf);
    }
}
