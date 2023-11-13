use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut file = File::open("examples/test.md").await?;
    let mut str: String = String::new();
    file.read_to_string(&mut str).await?;
    dbg!(&str.len());
    let mut splits = str.split("\n## ");
    dbg!(splits.next());
    for it in splits {
        dbg!(&it[0..10]);
    }

    Ok(())
}
