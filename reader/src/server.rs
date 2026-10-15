use axum::Router;
use std::error::Error;
use std::io::Write;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let report_dir = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or("usage: report-server <report-directory>")?;
    if !report_dir.join("index.html").is_file() {
        return Err(format!("report index not found in {}", report_dir.display()).into());
    }

    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let address = listener.local_addr()?;
    println!("http://{address}");
    std::io::stdout().flush()?;

    let app = Router::new()
        .fallback_service(ServeDir::new(report_dir).append_index_html_on_directories(true));
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let mut line = String::new();
            if let Err(error) = BufReader::new(tokio::io::stdin())
                .read_line(&mut line)
                .await
            {
                eprintln!("could not read shutdown request: {error}");
            }
        })
        .await?;

    Ok(())
}
