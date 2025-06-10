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
        sender_count: AtomicUsize::new(1),
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
    /// Counter for tracking the number of senders.
    sender_count: AtomicUsize,
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
        loop {
            // Check if channel is closed before attempting to send
            if self.inner.is_closed.load(Ordering::Acquire) {
                return Err(ChannelError::ChannelClosed);
            }

            // Try to push the value to the channel
            match self.inner.buffer.push(value) {
                Ok(_) => {
                    // Only increment count after successfully pushing
                    let old_count = self.inner.count.fetch_add(1, Ordering::AcqRel);
                    // Notify one consumer if there are any waiting
                    if old_count == 0 {
                        self.inner.consumer_waiters.notify_one();
                    }
                    return Ok(());
                }
                Err(v) => {
                    // Channel is full, wait for space
                    value = v;
                    // Double-check if channel was closed while we were waiting
                    if self.inner.is_closed.load(Ordering::Acquire) {
                        return Err(ChannelError::ChannelClosed);
                    }
                    // Wait for space to become available
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
        tracing::debug!(
            "Closing channel, current count: {}",
            self.inner.count.load(Ordering::Relaxed)
        );
        let was_closed = self.inner.is_closed.swap(true, Ordering::Release);
        if was_closed {
            tracing::debug!("Channel was already closed");
            return;
        }

        // Notify all waiting producers and consumers so they can check the closed state
        tracing::debug!("Notifying all waiters");
        self.inner.producer_waiters.notify_waiters();
        self.inner.consumer_waiters.notify_waiters();
        tracing::debug!("Channel closed");
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
        self.max_capacity() - self.len()
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
        let old_count = self.inner.sender_count.fetch_sub(1, Ordering::Relaxed);
        if old_count == 1 {
            tracing::debug!("Last sender dropped, closing channel");
            self.close();
        } else {
            tracing::debug!("Sender dropped, {} senders remaining", old_count - 1);
        }
    }
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        self.inner.sender_count.fetch_add(1, Ordering::Relaxed);
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
            // Try to pop a value from the channel first
            if let Some(value) = self.inner.buffer.pop() {
                // Only decrement count after successfully popping
                let prev_count = self.inner.count.fetch_sub(1, Ordering::AcqRel);
                // Notify one producer that space is available
                if prev_count == self.inner.buffer.capacity() {
                    self.inner.producer_waiters.notify_one();
                }
                return Ok(Some(value));
            }

            // Channel is empty, check if it's closed
            if self.inner.is_closed.load(Ordering::Acquire) {
                // Double-check if there are any items that might have been added
                // after we checked but before we saw the closed flag
                if let Some(value) = self.inner.buffer.pop() {
                    let prev_count = self.inner.count.fetch_sub(1, Ordering::AcqRel);
                    if prev_count == self.inner.buffer.capacity() {
                        self.inner.producer_waiters.notify_one();
                    }
                    return Ok(Some(value));
                }
                return Ok(None);
            }

            // Wait for new items to arrive
            self.inner.consumer_waiters.notified().await;
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
        self.max_capacity() - self.len()
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
