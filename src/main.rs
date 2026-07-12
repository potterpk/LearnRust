use clap::Parser;
use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

#[derive(Debug)]
struct ScanResult {
    port: u16,
    banner: Option<String>,
}

async fn grab_banner(stream: &mut TcpStream, port: u16, timeout_ms: u64) -> Option<String> {
    // Send an HTTP probe on common web ports, raw newline otherwise
    let probe = match port {
        80 | 443 | 8080 | 8443 => b"GET / HTTP/1.0\r\n\r\n".as_ref(),
        _ => b"\r\n",
    };

    let duration = Duration::from_millis(timeout_ms);

    let _ = timeout(duration, stream.write_all(probe)).await;

    let mut buf = vec![0u8; 1024];
    match timeout(duration, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let raw = &buf[..n];
            let banner = String::from_utf8_lossy(raw)
                .trim()
                .chars()
                .filter(|c| !c.is_control() || *c == '\n')
                .collect::<String>();
            if banner.is_empty() {
                None
            } else {
                Some(banner)
            }
        }
        _ => None,
    }
}

async fn scan_port(target: &str, port: u16, timeout_ms: u64) -> Option<ScanResult> {
    let addr = format!("{}:{}", target, port);
    let socket_addr: SocketAddr = match addr.parse() {
        Ok(a) => a,
        Err(_) => return None,
    };

    let duration = Duration::from_millis(timeout_ms);
    match timeout(duration, TcpStream::connect(socket_addr)).await {
        Ok(Ok(mut stream)) => {
            let banner = grab_banner(&mut stream, port, timeout_ms).await;
            Some(ScanResult { port, banner })
        }
        _ => None,
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Scanning {} ports {}-{}", args.target, args.start, args.end);

    let mut handles = vec![];
    let mut open_ports: Vec<ScanResult> = vec![];

    for port in args.start..=args.end {
        let target = args.target.clone();
        let timeout_ms = args.timeout;

        let handle = tokio::spawn(async move {
            scan_port(&target, port, timeout_ms).await
        });

        handles.push(handle);

        if handles.len() >= args.concurrency {
            let results: Vec<_> = futures::future::join_all(handles.drain(..)).await;
            for result in results.into_iter().flatten().flatten() {
                open_ports.push(result);
            }
        }
    }

    let results: Vec<_> = futures::future::join_all(handles).await;
    for result in results.into_iter().flatten().flatten() {
        open_ports.push(result);
    }

    open_ports.sort_by_key(|r| r.port);

    if open_ports.is_empty() {
        println!("No open ports found.");
    } else if args.json {
        for r in &open_ports {
            let banner = r.banner.as_deref().unwrap_or("");
            println!(r#"{{"port":{},"banner":"{}"}}"#, r.port, banner);
        }
    } else {
        for r in &open_ports {
            match &r.banner {
                Some(b) => println!("  [OPEN] port {:5} | {}", r.port, b),
                None    => println!("  [OPEN] port {:5} | (no banner)", r.port),
            }
        }
    }

    println!("Done.");
}
