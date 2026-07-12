use clap::Parser;

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

fn main() {
    let args = Args::parse();
    println!("{:#?}", args);
}
