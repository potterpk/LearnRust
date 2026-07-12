use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

#[derive(Parser, Debug)]
#[command(name = "rscan", about = "A fast concurrent port scanner")]
struct Args {
    /// Target IP address or hostname
    target: String,

    /// Starting port
    #[arg(short, long, default_value_t = 1)]
    start: u16,

    /// Ending port
    #[arg(short, long, default_value_t = 1024)]
    end: u16,

    /// Number of concurrent tasks
    #[arg(short, long, default_value_t = 100)]
    concurrency: usize,

    /// Connection timeout in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    timeout: u64,

    /// Output as JSON
    #[arg(short, long)]
    json: bool,
}

async fn scan_port(target: &str, port: u16, timeout_ms: u64) -> Option<u16> {
    let addr = format!("{}:{}", target, port);
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(_) => return None,
    };

    let duration = Duration::from_millis(timeout_ms);
    match timeout(duration, TcpStream::connect(socket_addr)).await {
        Ok(Ok(_)) => Some(port),
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Scanning {} ports {}-{}", args.target, args.start, args.end);

    let mut handles = vec![];

    for port in args.start..=args.end {
        let target = args.target.clone();
        let timeout_ms = args.timeout;

        let handle = tokio::spawn(async move {
            scan_port(&target, port, timeout_ms).await
        });

        handles.push(handle);

        // throttle: only run `concurrency` tasks at a time
        if handles.len() >= args.concurrency {
            let results: Vec<_> = futures::future::join_all(handles.drain(..)).await;
            for result in results {
                if let Ok(Some(port)) = result {
                    println!("  [OPEN] port {}", port);
                }
            }
        }
    }

    // drain any remaining handles
    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results {
        if let Ok(Some(port)) = result {
            println!("  [OPEN] port {}", port);
        }
    }

    println!("Done.");
}
