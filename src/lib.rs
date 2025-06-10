//! # tokio-mpmc
//!
//! A high-performance multi-producer multi-consumer (MPMC) channel implementation based on Tokio.
//!
//! This library provides an efficient asynchronous MPMC channel that supports multiple producers
//! and consumers to process messages through pooling.
//!
//! ## Features
//!
//! - Asynchronous implementation based on Tokio
//! - Support for multi-producer multi-consumer pattern
//! - Message processing using consumer pool
//! - Simple and intuitive API
//! - Complete error handling
//! - Channel capacity control
//!
//! ## Usage Example
//!
//! ```rust
//! // Using the channel API (recommended)
//! use tokio_mpmc::channel;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a channel with capacity of 100
//!     let (tx, rx) = channel(100);
//!
//!     // Send a message
//!     if let Err(e) = tx.send("Hello").await {
//!         eprintln!("Send failed: {}", e);
//!     }
//!
//!     // Receive a message
//!     match rx.recv().await {
//!         Ok(Some(msg)) => println!("Received message: {}", msg),
//!         Ok(None) => println!("Channel is closed"),
//!         Err(e) => eprintln!("Receive failed: {}", e),
//!     }
//! }
//! ```
//!
//! The legacy Queue API is still available:
//!
//! ```rust
//! // Using the legacy Queue API
//! use tokio_mpmc::Queue;
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a queue with capacity of 100
//!     let queue = Queue::new(100);
//!
//!     // Send a message
//!     if let Err(e) = queue.send("Hello").await {
//!         eprintln!("Send failed: {}", e);
//!     }
//!
//!     // Receive a message
//!     match queue.receive().await {
//!         Ok(Some(msg)) => println!("Received message: {}", msg),
//!         Ok(None) => println!("Queue is empty"),
//!         Err(e) => eprintln!("Receive failed: {}", e),
//!     }
//!
//!     // Close the queue
//!     queue.close();
//! }
//! ```

mod channel;
mod errors;
mod queue;

pub use channel::{Receiver, Sender, channel};
pub use errors::{ChannelError, QueueError};
pub use queue::Queue;

/// Represents the result type for channel operations
pub type ChannelResult<T> = std::result::Result<T, ChannelError>;
/// Represents the result type for queue operations (legacy)
pub type QueueResult<T> = std::result::Result<T, QueueError>;
