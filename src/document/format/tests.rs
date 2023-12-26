use tower_lsp::lsp_types::Position;

use super::super::Document;
use super::*;

trait QuickEdit {
    fn edit<S: ToString>(
        &self,
        soff: usize,
        eoff: usize,
        new_text: S,
    ) -> TextEdit;
}

impl<T> QuickEdit for T
where
    T: LspAdapter,
{
    fn edit<S: ToString>(
        &self,
        soff: usize,
        eoff: usize,
        new_text: S,
    ) -> TextEdit {
        let start = self.offset_to_position(soff).unwrap();
        let end = self.offset_to_position(eoff).unwrap();
        TextEdit {
            range: Range { start, end },
            new_text: new_text.to_string(),
        }
    }
}

#[test]
fn test_format() -> anyhow::Result<()> {
    let src = r#"
# Section 1

Some really long content which need to be formatted to have newline after HERE and another line after previous HERE

---

## Section 2

Content of section 2...

### Subsection"#;

    let doc = Document::parse(src).unwrap();
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

    assert_eq!(
        Some(vec![TextEdit {
            range: Range {
                start: Position {
                    line: 3,
                    character: 78
                },
                end: Position {
                    line: 3,
                    character: 79
                }
            },
            new_text: "\n".to_string(),
        }]),
        doc.format(range)
    );

    Ok(())
}

#[test]
fn format_should_ignore_one_big_line() -> anyhow::Result<()> {
    let src = r#"somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisblahblahhaha1234567"#;
    let doc = Document::parse(src).unwrap();
    assert_eq!(
        Some(vec![]),
        doc.format(Range {
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
fn format_should_break_after_long_line() -> anyhow::Result<()> {
    let src = r#"somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = Document::parse(src).unwrap();
    assert_eq!(
        Some(vec![doc.edit(82, 83, "\n")]),
        doc.format(Range {
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
fn format_should_break_single_line_into_multiple() -> anyhow::Result<()> {
    let src = r#"a somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = Document::parse(src).unwrap();
    assert_eq!(
        Some(vec![doc.edit(1, 2, "\n"), doc.edit(84, 85, "\n"),]),
        doc.format(Range {
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
#[ignore = "currently this test is failing"]
fn format_should_remove_whitespace_at_the_beginning() -> anyhow::Result<()> {
    let src = r#" somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = Document::parse(src).unwrap();
    assert_eq!(
        Some(vec![doc.edit(0, 1, ""), doc.edit(83, 84, "\n"),]),
        doc.format(Range {
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
