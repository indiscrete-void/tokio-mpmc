//! # tokio-mpmc
//!
//! A high-performance multi-producer multi-consumer (MPMC) queue implementation based on Tokio.
//!
//! This library provides an efficient asynchronous MPMC queue that supports multiple producers
//! and consumers to process messages through pooling.
//!
//! ## Features
//!
//! - Asynchronous implementation based on Tokio
//! - Support for multi-producer multi-consumer pattern
//! - Message processing using consumer pool
//! - Simple and intuitive API
//! - Complete error handling
//! - Queue capacity control
//!
//! ## Usage Example
//!
//! ```rust
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
//!     queue.close().await;
//! }
//! ```

mod errors;
mod queue;

pub use errors::QueueError;
pub use queue::Queue;

/// Represents the result type for queue operations
pub type QueueResult<T> = std::result::Result<T, QueueError>;
