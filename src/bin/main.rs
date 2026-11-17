use embed_search::frontend;


#[tokio::main]
async fn main() -> anyhow::Result<()>{
    frontend::app::run();
    Ok(())
}
