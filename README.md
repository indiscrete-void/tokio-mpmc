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

|              | `tokio-mpmc-channel`          | `tokio-mpmc-queue`               | `flume`                            |
|:-------------|:------------------------------|:---------------------------------|:---------------------------------- |
| **`non-io`** | `65.96 us` (✅ **1.00x**)      | `166.24 us` (❌ *2.52x slower*)   | `780.75 us` (❌ *11.84x slower*)    |
| **`io`**     | `48.12 ms` (✅ **1.00x**)      | `50.64 ms` (✅ **1.05x slower**)  | `202.26 ms` (❌ *4.20x slower*)     |

> **Note**: `non-io` means no IO operation, `io` means IO operation.

> See [benchmark code](./benches/queue_benchmark.rs)

## License

This project is licensed under the Apache-2.0 License. See the [LICENSE](LICENSE) file for details.
