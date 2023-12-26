use std::borrow::Cow;

use tower_lsp::lsp_types::{Position, Range};

use super::*;

struct TestDoc<'a>(&'a str);

impl<'a> SliceAccess for TestDoc<'a> {
    fn slice<'b>(&'b self, r: std::ops::Range<usize>) -> Cow<'b, str> {
        Cow::Borrowed(&self.0[r])
    }
}

impl<'a> LspAdapter for TestDoc<'a> {
    fn offset_to_position(&self, offset: usize) -> Option<Position> {
        if offset > self.0.len() {
            return None;
        }

        let lines = self.0[0..offset].split("\n");

        let (line, last) =
            lines.fold((0u32, None), |(line, _), it| (line + 1, Some(it)));

        let Some(tmp) = last else { return None };

        Some(Position {
            line: line - 1,
            character: tmp.chars().count() as u32,
        })
    }

    fn position_to_offset(&self, position: &Position) -> Option<usize> {
        let mut ret = 0;
        for (a, b) in self.0.split("\n").enumerate() {
            if a == position.line as usize {
                if position.character == 0 {
                    return Some(ret);
                }
                let Some((pos, chr)) =
                    b.char_indices().nth((position.character - 1) as usize)
                else {
                    return None;
                };
                return Some(ret + pos + chr.len_utf8());
            }
            ret += b.len() + 1; // 1 for '\n'.
        }
        None
    }
}

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
    let doc = TestDoc(src);
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
fn format_should_break_after_long_line() {
    let src = r#"somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = TestDoc(src);
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
}

#[test]
fn format_should_break_single_line_into_multiple() -> anyhow::Result<()> {
    let src = r#"a somereallylongstringisnotabletoformattomultiplelinestheyshoujldkeptsinglelineasisb ahblahhaha1234567"#;
    let doc = TestDoc(src);
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
    let doc = TestDoc(src);
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
