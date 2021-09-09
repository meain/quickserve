use std::fs;
use std::path::Path;

use argh::FromArgs;
use tide::{Middleware, Next, Request, Result};

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

struct Logger {
    log_level: String,
}

impl Logger {
    pub fn new(log_level: String) -> Self {
        Self { log_level }
    }
}
#[tide::utils::async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for Logger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> Result {
        let start = std::time::Instant::now();
        let path = req.url().path().to_owned();
        let method = req.method().to_string();
        let res = next.run(req).await;
        let status = res.status();
        if self.log_level == "all" || (self.log_level == "error" && !status.is_success()) {
            if status.is_success() {
                println!("{} {} {} {:?}", method, path, status, start.elapsed());
            } else {
                println!(
                    "{} {} {} {:?} {}",
                    method,
                    path,
                    status,
                    start.elapsed(),
                    status.canonical_reason()
                );
            }
        }
        Ok(res)
    }
}

async fn index_file(_: Request<()>) -> tide::Result {
    let index_file = Path::new(".").join("index.html");
    Ok(fs::read_to_string(index_file)
        .map_err(|_| {
            tide::Error::from_str(tide::StatusCode::NotFound, "Unable to find index file")
        })?
        .into())
}

#[async_std::main]
async fn main() -> Result<()> {
    let opts: Opts = argh::from_env();
    println!("[quickesrve] serving '{}' on port {}", opts.dir, opts.port);
    println!("> http://{}:{}", opts.host, opts.port);
    let mut app = tide::new();
    app.with(Logger::new(opts.loglevel));
    app.at("/").get(index_file);
    app.at("/").serve_dir(&opts.dir)?;
    app.listen(format!("{}:{}", opts.host, opts.port)).await?;
    Ok(())
}
