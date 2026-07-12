use clap::Parser;
use serde::Serialize;
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

    /// Specific ports to scan, comma-separated (e.g. 22,80,443) — overrides --start/--end
    #[arg(short, long, value_delimiter = ',')]
    ports: Option<Vec<u16>>,

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

#[derive(Debug, Serialize)]
struct ScanResult {
    port: u16,
    banner: Option<String>,
}

async fn resolve_host(target: &str) -> Option<String> {
    let addr = format!("{}:0", target);
    match tokio::net::lookup_host(&addr).await {
        Ok(mut addrs) => addrs.next().map(|a| a.ip().to_string()),
        Err(_) => None,
    }
}

async fn grab_banner(stream: &mut TcpStream, port: u16, timeout_ms: u64) -> Option<String> {
    let probe = match port {
        80 | 443 | 8080 | 8443 => b"GET / HTTP/1.0\r\n\r\n".as_ref(),
        _ => b"\r\n",
    };

    let duration = Duration::from_millis(timeout_ms);
    let _ = timeout(duration, stream.write_all(probe)).await;

    let mut buf = vec![0u8; 1024];
    match timeout(duration, stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let banner = String::from_utf8_lossy(&buf[..n])
                .trim()
                .chars()
                .filter(|c| !c.is_control() || *c == '\n')
                .collect::<String>();
            if banner.is_empty() { None } else { Some(banner) }
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

    let ip = match resolve_host(&args.target).await {
        Some(ip) => {
            if ip != args.target {
                println!("Resolved {} -> {}", args.target, ip);
            }
            ip
        }
        None => {
            eprintln!("Error: could not resolve host '{}'", args.target);
            std::process::exit(1);
        }
    };

    let ports: Vec<u16> = match &args.ports {
        Some(p) => p.clone(),
        None => (args.start..=args.end).collect(),
    };

    println!("Scanning {} ({} ports)", args.target, ports.len());

    let mut handles = vec![];
    let mut open_ports: Vec<ScanResult> = vec![];

    for port in ports {
        let target = ip.clone();
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
        println!("{}", serde_json::to_string_pretty(&open_ports).unwrap());
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
