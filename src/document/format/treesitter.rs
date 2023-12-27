#![allow(dead_code)]

use tree_sitter::{Node, Parser, TreeCursor, Tree};

fn debug_walk(mut cursor: TreeCursor) {
    loop {
        println!(
            "{:?}: {:?} {} {}",
            cursor.field_name(),
            cursor.node(),
            cursor.node().kind(),
            cursor.node().kind_id()
        );
        if !cursor.goto_first_child() {
            while !cursor.goto_next_sibling() {
                if !cursor.goto_parent() {
                    return;
                }
            }
        }
    }
}

pub struct Running<'a>(TreeCursor<'a>);

impl<'a> Running<'a> {
    fn step_into(&mut self) -> bool {
        if !self.0.goto_first_child() {
            return self.step_over();
        }
        return true;
    }

    fn step_over(&mut self) -> bool {
        while !self.0.goto_next_sibling() {
            if !self.0.goto_parent() {
                return false;
            }
        }
        return true;
    }

    fn is_match(&self) -> bool {
        let node = self.0.node();
        node.kind() == "paragraph" || node.kind() == "list"
    }
}

pub enum Traversal<'a> {
    Running(Running<'a>),
    Finished,
}

impl<'a> Traversal<'a> {
    pub fn from_cursor<'b>(cursor: TreeCursor<'b>) -> Traversal<'b> {
        Traversal::Running(Running(cursor))
    }
    
    fn step_into(&mut self) -> bool {
        match self {
            Traversal::Running(cursor) => cursor.step_into(),
            Traversal::Finished => false,
        }
    }

    fn step_over(&mut self) -> bool {
        match self {
            Traversal::Running(cursor) => cursor.step_over(),
            Traversal::Finished => false,
        }
    }

    fn is_match(&self) -> bool {
        match self {
            Traversal::Running(cursor) => cursor.is_match(),
            Traversal::Finished => false,
        }
    }
}

impl<'a> Iterator for Traversal<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Traversal::Running(cursor) => {
                while !cursor.is_match() {
                    if !cursor.step_into() {
                        *self = Traversal::Finished;
                        return None;
                    }
                }
                let ret = Some(cursor.0.node());
                if !cursor.step_over() {
                    *self = Traversal::Finished;
                }
                return ret;
            },
            Traversal::Finished => {
                return None;
            },
        };
    }
}

fn simple(buf: &[u8]) {
    let lang = tree_sitter_md::language();
    let mut parser = Parser::new();
    parser.set_language(lang).expect("should set lang");
    let tree = parser.parse(buf, None).expect("should parse markdown doc");

    let mut node = tree
        .root_node()
        .descendant_for_byte_range(10, 10)
        .expect("should parse");
    while node != tree.root_node() {
        if node.kind() == "paragraph" {
            println!("Found paragraph: {:?}", &node);
            break;
        }
        if node.kind() == "section" || node.kind() == "document" {
            println!("Oh no that's not what I wanted");
            break;
        }
        node = node.parent().expect("should have parent");
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
#[ignore = "debug purpose only"]
fn debug_walk_should_work() {
    let tree = tree(BUF);
    debug_walk(tree.walk());
}

#[test]
fn simple_should_work() {
    simple(BUF);
}

#[test]
fn traversal_should_work() {
    let tree = tree(BUF);
    let mut trav = Traversal::from_cursor(tree.walk());
    
    while let Some(it) = trav.next() {
        dbg!(it);
    }
}
