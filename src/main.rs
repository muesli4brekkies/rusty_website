use rusty_website::types::Result;
#[tokio::main]
async fn main() -> Result<()> {
    rusty_website::server::run::start_server().await?;
    Ok(())
}
