use tower_lsp::lsp_types::{Position, Range};

use super::*;
use crate::document::util::{QuickEdit as _, TestDoc};

#[test]
fn test_format() {
    let src = r#"
# Section 1

Some really long content which need to be formatted to have newline after HERE and another line after previous HERE

---

## Section 2

Content of section 2...

### Subsection"#;

    let doc = TestDoc(src);
    let range = Range {
        start: Position {
            line: 1,
            character: 0,
        },
        end: Position {
            line: 7,
            character: 0,
        },
    };

    let expected = src.to_string().replace("HERE and", "HERE\nand");

    assert_eq!(
        expected,
        doc.apply_edits(Formatter::new(&doc).format(range).unwrap())
    );
}

#[test]
fn format_should_ignore_one_big_line() {
    let src = r#"somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisblahblahhaha1234567"#;
    let doc = TestDoc(src);
    assert_eq!(
        src,
        doc.apply_edits(
            Formatter::new(&doc)
                .format(Range {
                    start: Position {
                        line: 0,
                        character: 0
                    },
                    end: Position {
                        line: 0,
                        character: 100
                    }
                })
                .unwrap()
                .as_slice()
        )
    );
}

#[test]
fn format_should_break_after_long_line() {
    let src = r#"somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = TestDoc(src);
    assert_eq!(
        src.to_string().replace("isb ahblah", "isb\nahblah"),
        doc.apply_edits(
            Formatter::new(&doc)
                .format(Range {
                    start: Position {
                        line: 0,
                        character: 0
                    },
                    end: Position {
                        line: 0,
                        character: 100
                    }
                })
                .unwrap()
        )
    );
}

#[test]
fn format_should_break_single_line_into_multiple() -> anyhow::Result<()> {
    let src = r#"a somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = TestDoc(src);
    assert_eq!(
        "a\nsomereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb\nahblahhaha1234567",
        doc.apply_edits(Formatter::new(&doc).format(Range {
            start: Position {
                line: 0,
                character: 0
            },
            end: Position {
                line: 0,
                character: 100
            }
        }).unwrap())
    );
    Ok(())
}

#[test]
fn format_should_keep_line_breaks() -> anyhow::Result<()> {
    let src = "Paragraph  \n  with  \nline break\n";
    let doc = TestDoc(src);
    let range = Range {
        start: Position {
            line: 0,
            character: 0,
        },
        end: Position {
            line: src.lines().count() as u32,
            character: 0,
        },
    };
    assert_eq!(
        "Paragraph  \nwith  \nline break\n",
        doc.apply_edits(&Formatter::new(&doc).format(range).unwrap())
    );
    Ok(())
}

#[test]
#[ignore = "currently this test is failing"]
fn format_should_remove_whitespace_at_the_beginning() -> anyhow::Result<()> {
    let src = r#" somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = TestDoc(src);
    assert_eq!(
        Some(vec![doc.edit(0, 1, ""), doc.edit(83, 84, "\n"),]),
        Formatter::new(&doc).format(Range {
            start: Position {
                line: 0,
                character: 0
            },
            end: Position {
                line: 0,
                character: 100
            }
        })
    );
    Ok(())
}

#[test]
fn process_section_should_format_properly() {
    assert_eq!(
        "a\nsomereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb\nahblahhaha1234567", 
        process_section(r#"a somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#)
    );
}

#[test]
fn process_section_should_format_url() {
    assert_eq!(
        "a:\nhttps://someurl.com\nahblahhaha1234567",
        process_section(r#"a: https://someurl.com ahblahhaha1234567"#)
    );
}

#[test]
fn process_section_bullet_items() {
    let src = r#"
- item 1
- item 2
  - subitem
- item 3  
  multiline content
- item 4: https://someurl.com is good
- item 5"#
        .trim();
    assert_eq!(
        r#"
- item 1
- item 2
  - subitem
- item 3  
  multiline content
- item 4:
  https://someurl.com
  is good
- item 5"#
            .trim(),
        process_section(src)
    );
}
