#[tokio::main]
async fn main() -> anyhow::Result<()> {
	directory_browser::run().await
}
