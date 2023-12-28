use std::ops::Range;

use tower_lsp::lsp_types::{Range as LspRange, TextEdit};
use tree_sitter::{Node, Parser, Tree};

use crate::document::format::{process_section, process_list_items};

use super::treesitter::{Traversal, debug_walk};
use super::util::TestDoc;
use super::{LspAdapter, LspRangeFormat, SliceAccess};

struct Tmp<'a, T: LspAdapter + SliceAccess> {
    buf: &'a T,
    tree_root: Node<'a>,
}

impl<'a, T: LspAdapter + SliceAccess> Tmp<'a, T> {
    fn range_from_lsp(&self, range: LspRange) -> Range<usize> {
        self.buf.position_to_offset(&range.start).unwrap()..
            self.buf.position_to_offset(&range.end).unwrap()
    }

    fn fmt<'b, T2: Iterator<Item = Node<'b>>>(&self, nodes: T2) {
        for it in nodes {
            dbg!(it);
        }
    }
}

fn process_list_item_node(node: Node<'_>) {
    if node.kind() != "list_item" {
        return;
    }
}

fn process_list_node(node: Node<'_>) {
    let mut cursor = node.walk();
    cursor.goto_first_child();
    loop {
        process_list_item_node(cursor.node());
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

impl<'a, T: LspAdapter + SliceAccess> LspRangeFormat for Tmp<'a, T> {
    fn format(
        &self,
        range: LspRange,
    ) -> Option<Vec<tower_lsp::lsp_types::TextEdit>> {
        let r2 = self.range_from_lsp(range);
        let cursor = self.tree_root.walk();
        let trav = Traversal::from_cursor(cursor);
        for it in trav {
            let r1 = it.byte_range();
            if r1.start >= r2.end || r2.start >= r1.end {
                continue;
            }
            
            match it.kind() {
                "paragraph" => {
                    // dbg!(&self.buf.slice(r1.clone()));
                    let mut updated = process_section(&self.buf.slice(r1));
                    updated.push('\n');
                    // dbg!(updated);
                },
                "list" => {
                    process_list_node(it);
                    // dbg!(&self.buf.slice(r1.clone()));
                    // let updated = process_list_items(&self.buf.slice(r1));
                    // dbg!(updated);
                },
                _ => panic!("unexpected type: {}", it.kind()),
            }
        }

        None
    }
}

fn tree(buf: &[u8]) -> Tree {
    let lang = tree_sitter_md::language();
    let mut parser = Parser::new();
    parser.set_language(lang).expect("should set lang");

    parser.parse(buf, None).expect("should parse markdown doc")
}

// TODO: Add mod tests with test configuration. and put all the tests in it.

const BUF: &'static [u8] = br#"Title
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
    let tree = tree(BUF);
    let node = tree.root_node();
    let buf = String::from_utf8_lossy(BUF);
    let doc = TestDoc(&buf);

    let tmp = Tmp {
        buf: &doc,
        tree_root: node,
    };

    tmp.format(LspRange {
        start: doc.offset_to_position(0).unwrap(),
        end: doc.offset_to_position(BUF.len()).unwrap(),
    });
}
