# tokio-mpmc

[![Crates.io](https://img.shields.io/crates/v/tokio-mpmc.svg)](https://crates.io/crates/tokio-mpmc)
[![Documentation](https://docs.rs/tokio-mpmc/badge.svg)](https://docs.rs/tokio-mpmc)
[![License: Apache 2.0](https://img.shields.io/badge/License-Apache2.0-yellow.svg)](https://opensource.org/license/apache-2-0)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/lispking/tokio-mpmc/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/lispking/tokio-mpmc/actions?query=branch%3Amain)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/lispking/tokio-mpmc)

A high-performance multi-producer multi-consumer (MPMC) queue implementation based on Tokio.

![architecture](docs/architecture.png)

## Features

- Asynchronous implementation based on Tokio
- Support for multi-producer multi-consumer pattern
- Message processing using consumer pool
- Simple and intuitive API
- Complete error handling
- Queue capacity control

## Installation

Add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
tokio-mpmc = "0.2"
```

## Usage Example

### Queue

```rust
use tokio_mpmc::Queue;

#[tokio::main]
async fn main() {
    // Create a queue with capacity of 100
    let queue = Queue::new(100);

    // Send a message
    if let Err(e) = queue.send("Hello").await {
        eprintln!("Send failed: {}", e);
    }

    // Receive a message
    match queue.receive().await {
        Ok(Some(msg)) => println!("Received message: {}", msg),
        Ok(None) => println!("Queue is empty"),
        Err(e) => eprintln!("Receive failed: {}", e),
    }

    // Close the queue
    drop(queue);
}
```

### Channel

```rust
use tokio_mpmc::channel;

#[tokio::main]
async fn main() {
    // Create a channel with capacity of 100
    let (tx, rx) = channel(100);

    // Send a message
    if let Err(e) = tx.send("Hello").await {
        eprintln!("Send failed: {}", e);
    }

    // Receive a message
    match rx.recv().await {
        Ok(Some(msg)) => println!("Received message: {}", msg),
        Ok(None) => println!("Channel is closed"),
        Err(e) => eprintln!("Receive failed: {}", e),
    }

    // Close the channel
    drop(tx);
}
```

## Performance

```bash
cargo criterion --message-format=json | criterion-table > BENCHMARKS.md
```

### Benchmark Results

|              | `tokio-mpsc-channel`          | `tokio-mpmc-channel`             | `tokio-mpmc-queue`               | `flume`                           |
|:-------------|:------------------------------|:---------------------------------|:---------------------------------|:--------------------------------- |
| **`non-io`** | `1.39 ms` (✅ **1.00x**)       | `65.38 us` (🚀 **21.21x faster**) | `168.86 us` (🚀 **8.21x faster**) | `773.68 us` (✅ **1.79x faster**)  |
| **`io`**     | `197.97 ms` (✅ **1.00x**)     | `46.32 ms` (🚀 **4.27x faster**)  | `46.83 ms` (🚀 **4.23x faster**)  | `197.76 ms` (✅ **1.00x faster**)  |

> **Note**: `non-io` means no IO operation, `io` means IO operation.

> See [benchmark code](./benches/queue_benchmark.rs)

## License

This project is licensed under the Apache-2.0 License. See the [LICENSE](LICENSE) file for details.
