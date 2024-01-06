use std::iter;
use std::ops::Range;

use tower_lsp::lsp_types::{
    Position as LspPosition, Range as LspRange, TextEdit,
};
use tree_sitter::{Node, Parser, Point, Tree};

use super::treesitter::Traversal;
use super::{
    process_list_items, process_section, LspAdapter, LspRangeFormat,
    SliceAccess,
};

pub struct Formatter<'a, T: LspAdapter + SliceAccess> {
    buf: &'a T,
    tree: Tree,
}

impl<'a, T: LspAdapter + SliceAccess> Formatter<'a, T> {
    pub fn new(buf: &'a T) -> Self {
        let lang = tree_sitter_md::language();
        let mut parser = Parser::new();
        parser.set_language(lang).expect("should set lang");

        // FIXME: Currently tree is built from scratch, but it can be built
        // incrementally.
        let tree = parser
            .parse(buf.slice(0..).as_bytes(), None)
            .expect("should parse markdown doc");

        Self { buf, tree }
    }

    fn range_from_lsp(&self, range: LspRange) -> Range<usize> {
        self.buf.position_to_offset(&range.start).unwrap()..
            self.buf.position_to_offset(&range.end).unwrap()
    }
}

/// Simple utility to convert between LSP and treesitter positions.
struct MyPosition(LspPosition);

impl From<Point> for MyPosition {
    fn from(value: Point) -> Self {
        Self(LspPosition {
            line: value.row as u32,
            character: value.column as u32,
        })
    }
}

impl Into<LspPosition> for MyPosition {
    fn into(self) -> LspPosition {
        self.0
    }
}

fn process_list_node<T: SliceAccess>(
    buf: &T,
    node: Node<'_>,
) -> impl Iterator<Item = TextEdit> {
    let r1 = node.byte_range();
    let src = buf.slice(r1.clone());
    let src2 = src.trim_end();
    let src3 = &src[src2.len()..];

    let mut ret = process_list_items(src2);
    ret.push_str(src3);

    iter::once(TextEdit {
        range: LspRange {
            start: MyPosition::from(node.start_position()).into(),
            end: MyPosition::from(node.end_position()).into(),
        },
        new_text: ret,
    })
}

impl<'a, T: LspAdapter + SliceAccess> LspRangeFormat for Formatter<'a, T> {
    fn format(
        &self,
        range: LspRange,
    ) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
        let r2 = self.range_from_lsp(range);
        let cursor = self.tree.root_node().walk();
        let trav = Traversal::from_cursor(cursor);
        let mut res: Box<dyn Iterator<Item = TextEdit>> =
            Box::new(iter::empty());
        for it in trav {
            let r1 = it.byte_range();
            if r1.start >= r2.end || r2.start >= r1.end {
                continue;
            }

            match it.kind() {
                "paragraph" => {
                    let src = self.buf.slice(r1.clone());
                    let src2 = src.trim_end();
                    let src3 = &src[src2.len()..];
                    let mut updated = process_section(&src2);
                    updated.push_str(src3);
                    res =
                        Box::new(
                            res.chain(iter::once(TextEdit {
                                range: LspRange {
                                    start: MyPosition::from(
                                        it.start_position(),
                                    )
                                    .into(),
                                    end: MyPosition::from(it.end_position())
                                        .into(),
                                },
                                new_text: updated,
                            })),
                        );
                },
                "list" => {
                    res = Box::new(res.chain(process_list_node(self.buf, it)));
                    // dbg!(&self.buf.slice(r1.clone()));
                    // let updated = process_list_items(&self.buf.slice(r1));
                    // dbg!(updated);
                },
                _ => panic!("unexpected type: {}", it.kind()),
            }
        }

        Some(res.collect())
    }
}

// TODO: Add mod tests with test configuration. and put all the tests in it.

#[allow(dead_code)]
const BUF: &'static str = r#"Title
---

Some paragraph here, with long text longer than 80 characters, need some reformatting to align 80 cols.

Also the paragraph has a https://example.com link, which I'd like to see it placed in a sole line.
                               
a Sometimesthereisalonglinewithmorethan80characterssowecannotformatthisproperlybutstill... b

Line breaking after two space  
must be kept.

-   List is not an exception, and should be formatted to 80
-   Multiple  
    lines
-   Some paragraph here, with long text longer than 80 characters, need some reformatting to align 80 cols.
    -   https://example.com
    -   When https://example.com is in the middle of paragraph...
-   Sometimesthereisalonglinewithmorethan80characterssowecannotformatthisproperlybutstill...

Some other paragraph here

```markdown
Some other markdown
```

```cpp
#include <iostream>

using namespace std;
```
"#;

#[test]
fn format_should_work() {
use crate::document::incremental_sync::IncrementalSync as _;
use crate::document::test_doc::TestDoc;

    let doc = TestDoc::new(BUF);

    let tmp = Formatter::new(&doc);

    let edits = tmp
        .format(LspRange {
            start: doc.offset_to_position(0).unwrap(),
            end: doc.offset_to_position(BUF.len()).unwrap(),
        })
        .unwrap();

    println!("{:?}", doc.apply_edits(&edits));
}
