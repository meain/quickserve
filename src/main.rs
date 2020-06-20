use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tide::http::mime;
use tide::{After, Before};
use tide::{Middleware, Next, Request, Response, Result, StatusCode};

// use crate::log;
// use crate::utils::BoxFuture;
// use crate::{Middleware, Next, Request};


pub(crate) type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(Debug, Default, Clone)]
pub struct LogMiddleware {
    _priv: (),
}

impl LogMiddleware {
    /// Create a new instance of `LogMiddleware`.
    #[must_use]
    pub fn new() -> Self {
        Self { _priv: () }
    }

    /// Log a request and a response.
    async fn log<'a, State: Send + Sync + 'static>(
        &'a self,
        ctx: Request<State>,
        next: Next<'a, State>,
    ) -> crate::Result {
        let path = ctx.url().path().to_owned();
        let method = ctx.method().to_string();
        // log::info!("<-- Request received", {
        //     method: method,
        //     path: path,
        // });
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
                    err.to_string(),
                    method,
                    path,
                    err.status(),
                    format!("{:?}", start.elapsed()),
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
    let mut app = tide::new();
    app.middleware(LogMiddleware::new());
    app.at("/").serve_dir(".")?;
    app.listen(format!("127.0.0.1:{}", port)).await?;
    Ok(())
}
