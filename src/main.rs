use argh::FromArgs;
use std::future::Future;
use std::pin::Pin;
use tide::{Middleware, Next, Request, Result};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

#[derive(FromArgs)]
/// Quickly serve local static files
struct Opts {
    /// port to serve at
    #[argh(option, short = 'p', default = "8080")]
    port: usize,

    /// host to serve at
    #[argh(option, short = 'h', default = "String::from(\"localhost\")")]
    host: String,

    /// level of log output (none, error, all)
    #[argh(option, default = "String::from(\"all\")")]
    loglevel: String,

    /// directory to be served, uses '.' by default
    #[argh(positional, default = "String::from(\".\")")]
    dir: String,
}

#[derive(Debug, Default, Clone)]
struct LogMiddleware {
    loglevel: String,
}

impl LogMiddleware {
    pub fn new(loglevel: String) -> Self {
        Self { loglevel }
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
                if self.loglevel == "all" {
                    println!(
                        "{} {} {} {}",
                        method,
                        path,
                        status,
                        format!("{:?}", start.elapsed()),
                    );
                }
                Ok(res)
            }
            Err(err) => {
                if self.loglevel == "all" || self.loglevel == "error" {
                    println!(
                        "{} {} {} {} {}",
                        method,
                        path,
                        err.status(),
                        format!("{:?}", start.elapsed()),
                        err.to_string(),
                    );
                }
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
    let opts: Opts = argh::from_env();
    println!("[quickesrve] serving '{}' on port {}", opts.dir, opts.port);
    println!("> http://{}:{}", opts.host, opts.port);
    let mut app = tide::new();
    app.middleware(LogMiddleware::new(opts.loglevel));
    app.at("/").serve_dir(opts.dir)?;
    app.listen(format!("{}:{}", opts.host, opts.port)).await?;
    Ok(())
}
