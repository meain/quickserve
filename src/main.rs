use std::future::Future;
use std::pin::Pin;
use tide::{Middleware, Next, Request, Result};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Default, Clone)]
struct LogMiddleware;

impl LogMiddleware {
    pub fn new() -> Self {
        Self
    }

    async fn log<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result {
        let path = ctx.url().path().to_owned();
        let method = ctx.method().to_string();
        let start = std::time::Instant::now();
        match next.run(ctx).await {
            Ok(res) => {
                let status = res.status();
                println!(
                    "{} {} {} {}",
                    method,
                    path,
                    status,
                    format!("{:?}", start.elapsed()),
                );
                Ok(res)
            }
            Err(err) => {
                println!(
                    "{} {} {} {} {}",
                    method,
                    path,
                    err.status(),
                    format!("{:?}", start.elapsed()),
                    err.to_string(),
                );
                Err(err)
            }
        }
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for LogMiddleware {
    fn handle<'a>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, crate::Result> {
        Box::pin(async move { self.log(ctx, next).await })
    }
}

#[async_std::main]
async fn main() -> Result<()> {
    let port = std::env::args().nth(1).unwrap_or("8080".to_string());
    let dir = std::env::args().nth(2).unwrap_or(".".to_string());
    if port == "--help" || port == "-h" {
        println!("quickserve: Quicky serve a dir");
        println!("Usage: quickserve <port> <dir>");
        return Ok(())
    }
    println!("[quickesrv] serving '{}' on port {}", dir, port);
    let mut app = tide::new();
    app.middleware(LogMiddleware::new());
    app.at("/").serve_dir(dir)?;
    app.listen(format!("127.0.0.1:{}", port)).await?;
    Ok(())
}
