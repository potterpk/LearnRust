# rscan

A fast, concurrent TCP port scanner with service banner grabbing, written in Rust.

> I'm currently learning Rust and built this project to understand async programming, ownership, and systems-level networking. This is one of my first Rust projects.

## Features

- Concurrent async scanning using Tokio — scans hundreds of ports in parallel without OS thread overhead
- Banner grabbing — connects to open ports and reads service responses to identify what's running
- Hostname resolution — accepts both IP addresses and domain names
- Flexible targeting — scan a port range (`--start`/`--end`) or specific ports (`--ports 22,80,443`)
- JSON output — machine-readable output for piping into other tools
- Configurable concurrency and timeout

## Usage

```
rscan [OPTIONS] <TARGET>

Arguments:
  <TARGET>  Target IP address or hostname

Options:
  -s, --start <START>        Starting port [default: 1]
  -e, --end <END>            Ending port [default: 1024]
  -p, --ports <PORTS>        Specific ports, comma-separated (overrides --start/--end)
  -c, --concurrency <N>      Number of concurrent tasks [default: 100]
  -t, --timeout <MS>         Connection timeout in milliseconds [default: 1000]
  -j, --json                 Output as JSON
  -h, --help                 Print help
```

## Examples

Scan default ports 1-1024:
```bash
rscan 192.168.1.1
```

Scan a specific range with higher concurrency:
```bash
rscan 192.168.1.1 --start 1 --end 10000 --concurrency 500 --timeout 500
```

Scan specific ports:
```bash
rscan 192.168.1.1 --ports 22,80,443,8080,8443
```

Scan by hostname with JSON output:
```bash
rscan example.com --ports 22,80,443 --json
```

## Build

Requires Rust 1.70+.

```bash
# debug build
cargo build

# optimized release build (recommended for actual scanning)
cargo build --release
./target/release/rscan <target>
```

## What I Learned

- **Async/await with Tokio** — why I/O-bound tasks like port scanning benefit from async over OS threads
- **Ownership and borrowing** — why `.clone()` is needed when moving data into async tasks
- **Traits** — how `AsyncReadExt`/`AsyncWriteExt` add methods to `TcpStream`, and how `#[derive(Serialize)]` generates JSON serialization
- **Error handling with `Result` and `Option`** — no null pointer exceptions, the compiler forces you to handle every failure case
- **Procedural macros** — how `#[derive(Parser)]` generates CLI parsing code from a struct at compile time

## Dependencies

- [`tokio`](https://tokio.rs) — async runtime
- [`clap`](https://docs.rs/clap) — CLI argument parsing
- [`serde`](https://serde.rs) / [`serde_json`](https://docs.rs/serde_json) — JSON serialization
- [`futures`](https://docs.rs/futures) — utilities for joining async tasks
