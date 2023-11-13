use lsp_md::Backend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = tokio::task::spawn_blocking(|| {
        LspService::build(|client| Backend::new(client)).finish()
    })
    .await?;

    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
