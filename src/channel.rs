use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crossbeam_queue::ArrayQueue;
use tokio::sync::Notify;

use crate::{ChannelError, ChannelResult};

/// A multi-producer, multi-consumer (MPMC) channel for communicating between
/// asynchronous tasks.
///
/// Multiple producers and consumers can send and receive values through the channel.
/// It uses a fixed-size buffer and provides backpressure by waiting when the channel is full
/// (for producers) or empty (for consumers).
///
/// # Examples
///
/// ```rust
/// use tokio_mpmc::channel;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a channel with capacity of 100
///     let (tx, rx) = channel(100);
///
///     // Send a message
///     if let Err(e) = tx.send("Hello").await {
///         eprintln!("Send failed: {}", e);
///     }
///
///     // Receive a message
///     match rx.recv().await {
///         Ok(Some(msg)) => println!("Received message: {}", msg),
///         Ok(None) => println!("Channel is closed"),
///         Err(e) => eprintln!("Receive failed: {}", e),
///     }
///
///     // Close the channel
///     drop(tx);
/// }
/// ```
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let inner = Arc::new(Inner {
        buffer: ArrayQueue::new(capacity),
        is_closed: AtomicBool::new(false),
        count: AtomicUsize::new(0),
        producer_waiters: Arc::new(Notify::new()),
        consumer_waiters: Arc::new(Notify::new()),
    });

    let sender = Sender {
        inner: inner.clone(),
    };
    let receiver = Receiver { inner };

    (sender, receiver)
}

/// The inner state of the channel using lock-free data structures.
/// This allows multiple producers and consumers to safely access and modify the channel state
/// with minimal contention.
struct Inner<T> {
    /// The underlying buffer storing the channel elements using a lock-free queue.
    buffer: ArrayQueue<T>,
    /// A flag indicating whether the channel has been closed.
    /// Once closed, no more items can be sent, and `recv` will return `None` after the buffer is empty.
    is_closed: AtomicBool,
    /// Counter for tracking the number of items in the channel.
    count: AtomicUsize,
    /// Notifier for producers waiting for space in the buffer.
    /// Producers wait on this when the channel is full.
    producer_waiters: Arc<Notify>,
    /// Notifier for consumers waiting for items in the buffer.
    /// Consumers wait on this when the channel is empty.
    consumer_waiters: Arc<Notify>,
}

/// The sending half of the channel.
///
/// This half can be used to send values to the channel.
/// It can be cloned to send from multiple tasks.
pub struct Sender<T> {
    inner: Arc<Inner<T>>,
}

/// The receiving half of the channel.
///
/// This half can be used to receive values from the channel.
/// It can be cloned to receive from multiple tasks.
pub struct Receiver<T> {
    inner: Arc<Inner<T>>,
}

impl<T> Sender<T> {
    /// Sends a message `value` to the channel.
    ///
    /// If the channel is full, the calling task will be suspended until space becomes available
    /// or the channel is closed.
    ///
    /// # Arguments
    ///
    /// * `value` - The message to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was successfully sent.
    /// `Err(ChannelError::ChannelClosed)` if the channel was closed while waiting or before sending.
    pub async fn send(&self, mut value: T) -> ChannelResult<()> {
        // If the channel is closed, return error immediately
        if self.inner.is_closed.load(Ordering::Acquire) {
            return Err(ChannelError::ChannelClosed);
        }

        // Try to push the value to the channel
        loop {
            match self.inner.buffer.push(value) {
                Ok(_) => {
                    self.inner.count.fetch_add(1, Ordering::Release);
                    self.inner.consumer_waiters.notify_one();
                    return Ok(());
                }
                Err(v) => {
                    // Channel is full, wait for space
                    if self.inner.is_closed.load(Ordering::Acquire) {
                        return Err(ChannelError::ChannelClosed);
                    }
                    value = v;
                    self.inner.producer_waiters.notified().await;
                }
            }
        }
    }

    /// Closes the channel.
    ///
    /// This prevents any new messages from being sent. Tasks currently waiting in `send`
    /// will return `Err(ChannelError::ChannelClosed)`. Tasks waiting in `recv` will return
    /// `Ok(None)` once the channel is empty.
    pub fn close(&self) {
        self.inner.is_closed.store(true, Ordering::Release);
        // Notify all waiting producers and consumers so they can check the closed state
        self.inner.producer_waiters.notify_waiters();
        self.inner.consumer_waiters.notify_waiters();
    }

    /// Gets the current number of elements in the channel.
    ///
    /// # Returns
    ///
    /// The number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.inner.count.load(Ordering::Acquire)
    }

    /// Checks if the channel is currently empty.
    ///
    /// # Returns
    ///
    /// `true` if the channel contains no elements, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if the channel is currently full.
    ///
    /// # Returns
    ///
    /// `true` if the number of elements equals the capacity, `false` otherwise.
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.buffer.capacity()
    }

    /// Checks if the channel has been closed.
    ///
    /// # Returns
    ///
    /// `true` if the channel is closed, `false` otherwise.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed.load(Ordering::Acquire)
    }

    /// Returns the current capacity of the channel.
    ///
    /// # Returns
    ///
    /// The current number of elements in the channel.
    pub fn capacity(&self) -> usize {
        self.len()
    }

    /// Returns the maximum buffer capacity of the channel.
    ///
    /// # Returns
    ///
    /// The maximum number of elements the channel can hold.
    pub fn max_capacity(&self) -> usize {
        self.inner.buffer.capacity()
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.close();
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Receiver<T> {
    /// Receives a message from the channel.
    ///
    /// If the channel is empty, the calling task will be suspended until an item becomes available
    /// or the channel is closed.
    ///
    /// # Returns
    ///
    /// `Ok(Some(value))` if a message was successfully received.
    /// `Ok(None)` if the channel is closed and empty.
    /// `Err(ChannelError::ChannelClosed)` if the channel was closed while waiting but not empty.
    pub async fn recv(&self) -> ChannelResult<Option<T>> {
        loop {
            // Try to pop a value from the channel
            match self.inner.buffer.pop() {
                Some(value) => {
                    self.inner.count.fetch_sub(1, Ordering::Release);
                    self.inner.producer_waiters.notify_one();
                    return Ok(Some(value));
                }
                None => {
                    // Channel is empty, check if it's closed
                    if self.inner.is_closed.load(Ordering::Acquire) {
                        return if self.inner.count.load(Ordering::Acquire) == 0 {
                            Ok(None)
                        } else {
                            Err(ChannelError::ChannelClosed)
                        };
                    }
                    // Wait for new items
                    self.inner.consumer_waiters.notified().await;
                }
            }
        }
    }

    /// Gets the current number of elements in the channel.
    ///
    /// # Returns
    ///
    /// The number of elements currently in the channel.
    pub fn len(&self) -> usize {
        self.inner.count.load(Ordering::Acquire)
    }

    /// Checks if the channel is currently empty.
    ///
    /// # Returns
    ///
    /// `true` if the channel contains no elements, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if the channel is currently full.
    ///
    /// # Returns
    ///
    /// `true` if the number of elements equals the capacity, `false` otherwise.
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.buffer.capacity()
    }

    /// Checks if the channel has been closed.
    ///
    /// # Returns
    ///
    /// `true` if the channel is closed, `false` otherwise.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed.load(Ordering::Acquire)
    }

    /// Returns the current capacity of the channel.
    ///
    /// # Returns
    ///
    /// The current number of elements in the channel.
    pub fn capacity(&self) -> usize {
        self.len()
    }

    /// Returns the maximum buffer capacity of the channel.
    ///
    /// # Returns
    ///
    /// The maximum number of elements the channel can hold.
    pub fn max_capacity(&self) -> usize {
        self.inner.buffer.capacity()
    }
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
