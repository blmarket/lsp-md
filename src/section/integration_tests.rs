use std::fs;
use super::Sections;

#[test]
fn test() -> anyhow::Result<()> {
  let contents = fs::read_to_string("test.md")
    .expect("Something went wrong reading the file");
  let sections = Sections::parse(&contents)?;
  
  dbg!(&sections.document[sections.sections[0].0.clone()]);
  dbg!(&sections.document[sections.sections[1].0.clone()]);
  dbg!(&sections.document[sections.sections[2].0.clone()]);
  
  Ok(())
}