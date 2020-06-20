#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    let port = std::env::args().nth(1).unwrap_or("8080".to_string());
    let mut app = tide::new();
    tide::log::with_level(tide::log::LevelFilter::Info);
    // app.middleware(tide::log::LogMiddleware::new());
    app.at("/").serve_dir(".")?;
    app.listen(format!("127.0.0.1:{}", port)).await?;
    Ok(())
}
