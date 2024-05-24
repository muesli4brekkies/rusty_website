use rusty_website::types::Result;
fn main() -> Result<()> {
  rusty_website::server::run::start_server()?;
  Ok(())
}
