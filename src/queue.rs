use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// Main implementation of a multi-producer, multi-consumer (MPMC) asynchronous queue.
///
/// This queue allows multiple asynchronous tasks to concurrently send and receive messages.
/// It uses a fixed-size buffer and provides backpressure by waiting when the queue is full
/// (for producers) or empty (for consumers).
///
/// # Examples
///
/// ```rust
/// // Using the legacy Queue API
/// use tokio_mpmc::Queue;
///
/// #[tokio::main]
/// async fn main() {
///     // Create a queue with capacity of 100
///     let queue = Queue::new(100);
///
///     // Send a message
///     if let Err(e) = queue.send("Hello").await {
///         eprintln!("Send failed: {}", e);
///     }
///
///     // Receive a message
///     match queue.receive().await {
///         Ok(Some(msg)) => println!("Received message: {}", msg),
///         Ok(None) => println!("Queue is empty"),
///         Err(e) => eprintln!("Receive failed: {}", e),
///     }
///
///     // Close the queue
///     drop(queue);
/// }
#[derive(Clone)]
pub struct Queue<T> {
    inner: Arc<Inner<T>>,
}

use crossbeam_queue::ArrayQueue;
use tokio::sync::Notify;

use crate::{QueueError, QueueResult};

/// The inner state of the queue using lock-free data structures.
/// This allows multiple producers and consumers to safely access and modify the queue state
/// with minimal contention.
struct Inner<T> {
    /// The underlying buffer storing the queue elements using a lock-free queue.
    buffer: ArrayQueue<T>,
    /// A flag indicating whether the queue has been closed.
    /// Once closed, no more items can be sent, and `receive` will return `None` after the buffer is empty.
    is_closed: AtomicBool,
    /// Counter for tracking the number of items in the queue.
    count: AtomicUsize,
    /// Notifier for producers waiting for space in the buffer.
    /// Producers wait on this when the queue is full.
    producer_waiters: Arc<Notify>,
    /// Notifier for consumers waiting for items in the buffer.
    /// Consumers wait on this when the queue is empty.
    consumer_waiters: Arc<Notify>,
}

impl<T> Queue<T> {
    /// Creates a new MPMC queue with the specified `capacity`.
    ///
    /// # Arguments
    ///
    /// * `capacity` - The maximum number of elements the queue can hold.
    ///
    /// # Returns
    ///
    /// A new `Queue` instance.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Inner {
                buffer: ArrayQueue::new(capacity),
                is_closed: AtomicBool::new(false),
                count: AtomicUsize::new(0),
                producer_waiters: Arc::new(Notify::new()),
                consumer_waiters: Arc::new(Notify::new()),
            }),
        }
    }

    /// Sends a message `value` to the queue.
    ///
    /// If the queue is full, the calling task will be suspended until space becomes available
    /// or the queue is closed.
    ///
    /// # Arguments
    ///
    /// * `value` - The message to send.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the message was successfully sent.
    /// `Err(QueueError::QueueClosed)` if the queue was closed while waiting or before sending.
    pub async fn send(&self, mut value: T) -> QueueResult<()> {
        // If the queue is closed, return error immediately
        if self.inner.is_closed.load(Ordering::Acquire) {
            return Err(QueueError::QueueClosed);
        }

        // Try to push the value to the queue
        loop {
            match self.inner.buffer.push(value) {
                Ok(_) => {
                    self.inner.count.fetch_add(1, Ordering::Release);
                    self.inner.consumer_waiters.notify_one();
                    return Ok(());
                }
                Err(v) => {
                    // Queue is full, wait for space
                    if self.inner.is_closed.load(Ordering::Acquire) {
                        return Err(QueueError::QueueClosed);
                    }
                    value = v;
                    self.inner.producer_waiters.notified().await;
                }
            }
        }
    }

    /// Receives a message from the queue.
    ///
    /// If the queue is empty, the calling task will be suspended until an item becomes available
    /// or the queue is closed.
    ///
    /// # Returns
    ///
    /// `Ok(Some(value))` if a message was successfully received.
    /// `Ok(None)` if the queue is closed and empty.
    /// `Err(QueueError::QueueClosed)` if the queue was closed while waiting but not empty.
    pub async fn receive(&self) -> QueueResult<Option<T>> {
        loop {
            // Try to pop a value from the queue
            match self.inner.buffer.pop() {
                Some(value) => {
                    self.inner.count.fetch_sub(1, Ordering::Release);
                    self.inner.producer_waiters.notify_one();
                    return Ok(Some(value));
                }
                None => {
                    // Queue is empty, check if it's closed
                    if self.inner.is_closed.load(Ordering::Acquire) {
                        return if self.inner.count.load(Ordering::Acquire) == 0 {
                            Ok(None)
                        } else {
                            Err(QueueError::QueueClosed)
                        };
                    }
                    // Wait for new items
                    self.inner.consumer_waiters.notified().await;
                }
            }
        }
    }

    /// Closes the queue.
    ///
    /// This prevents any new messages from being sent. Tasks currently waiting in `send`
    /// will return `Err(QueueError::QueueClosed)`. Tasks waiting in `receive` will return
    /// `Ok(None)` once the queue is empty.
    pub fn close(&self) {
        self.inner.is_closed.store(true, Ordering::Release);
        // Notify all waiting producers and consumers so they can check the closed state
        self.inner.producer_waiters.notify_waiters();
        self.inner.consumer_waiters.notify_waiters();
    }

    /// Gets the current number of elements in the queue.
    ///
    /// # Returns
    ///
    /// The number of elements currently in the queue.
    pub fn len(&self) -> usize {
        self.inner.count.load(Ordering::Acquire)
    }

    /// Checks if the queue is currently empty.
    ///
    /// # Returns
    ///
    /// `true` if the queue contains no elements, `false` otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if the queue is currently full.
    ///
    /// # Returns
    ///
    /// `true` if the number of elements equals the capacity, `false` otherwise.
    pub fn is_full(&self) -> bool {
        self.len() >= self.inner.buffer.capacity()
    }

    /// Checks if the queue has been closed.
    ///
    /// # Returns
    ///
    /// `true` if the queue is closed, `false` otherwise.
    pub fn is_closed(&self) -> bool {
        self.inner.is_closed.load(Ordering::Acquire)
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        self.close();
    }
}
