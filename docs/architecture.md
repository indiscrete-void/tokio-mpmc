# tokio-mpmc Technical Architecture Design & Usage Guide

## Overview

`tokio-mpmc` is a high-performance multi-producer multi-consumer (MPMC) queue implementation based on the Tokio asynchronous runtime. It aims to provide an efficient, non-blocking data transfer mechanism for asynchronous Rust applications, especially suitable for scenarios where multiple asynchronous tasks need to concurrently send and receive data.

Unlike traditional synchronous queues, `tokio-mpmc` leverages Tokio's asynchronous features to ensure that when the queue is full or empty, related async tasks are suspended rather than blocking threads, thus making better use of system resources.

## Background: Why tokio-mpmc?

In asynchronous programming, especially when building high-performance concurrent systems, data transfer between tasks is a core issue. There are various queue implementations in the Rust ecosystem, such as `std::sync::mpsc`, `tokio::sync::mpsc`, `tokio::sync::broadcast`, and `crossbeam-queue::ArrayQueue`. However, when you need an MPMC queue deeply integrated with the Tokio async runtime, using these queues alone may have some limitations:

- **`std::sync::mpsc`**: This is the synchronous MPSC queue provided by the Rust standard library. It blocks threads and is not suitable for async contexts. Using it in Tokio tasks can block the entire task or even the runtime.
- **`tokio::sync::mpsc`**: This is Tokio's async MPSC queue, supporting multiple producers and a single consumer. Although it's async, it's designed for the MPSC pattern and doesn't directly support multiple consumers, requiring extra synchronization or workarounds to achieve MPMC, increasing complexity.
- **`tokio::sync::broadcast`**: This is Tokio's broadcast queue, supporting multiple producers and consumers, but each sent message is received by all consumers. This suits broadcast scenarios but not typical queue scenarios where each message should be processed by only one consumer.
- **`crossbeam-queue::ArrayQueue`**: This is a high-performance lock-free MPMC queue, safe for use across threads. However, it's synchronous; when the queue is full or empty, `push` or `pop` operations block the current thread. Using it in async tasks also requires an extra async adaptation layer (e.g., with `tokio::sync::Notify`) to avoid blocking, which means developers must manually implement complex waiting and notification logic.

The design goal of `tokio-mpmc` is to solve these problems, providing an out-of-the-box, high-performance MPMC queue seamlessly integrated with the Tokio async runtime. Internally, it combines the efficient lock-free features of `crossbeam-queue::ArrayQueue` with the async waiting/notification mechanism of `tokio::sync::Notify`, encapsulating the complex async adaptation logic and offering a simple, intuitive async `send` and `receive` API, making MPMC data transfer in Tokio applications more convenient and efficient.

## What is an MPMC Queue?

MPMC (Multi-Producer Multi-Consumer) queue is a concurrent data structure that allows multiple producers to send data to the queue simultaneously and multiple consumers to receive data from the queue at the same time. Compared to traditional SPSC (Single-Producer Single-Consumer) or MPSC (Multi-Producer Single-Consumer) queues, MPMC queues offer greater flexibility and throughput in concurrent scenarios.

In concurrent programming, queues are often used to transfer data between different tasks or threads. An efficient MPMC queue implementation is crucial for building high-performance concurrent systems.

## Why a Tokio-based MPMC Queue?

In asynchronous programming, especially when using async runtimes like Tokio, we need an MPMC queue that integrates seamlessly with the async ecosystem. Traditional synchronous MPMC queues block async tasks, leading to performance degradation or deadlocks. Therefore, a Tokio-based async MPMC queue can better leverage async I/O and coroutine advantages, providing a non-blocking data transfer mechanism.

The `tokio-mpmc` library is designed for this purpose. It utilizes Tokio's async features to provide a high-performance, easy-to-use MPMC queue implementation.

## Features of tokio-mpmc

The `tokio-mpmc` library offers the following main features:

- **Async implementation based on Tokio**: Fully asynchronous, never blocks the Tokio runtime.
- **Supports MPMC mode**: Allows multiple async tasks as producers and consumers.
- **Queue capacity control**: Supports bounded queues to prevent unlimited memory growth.
- **Simple and intuitive API**: Provides easy-to-understand async methods like `send` and `receive`.
- **Comprehensive error handling**: Uses the `QueueError` enum to clearly indicate possible errors.

## How to Use tokio-mpmc

First, add `tokio-mpmc` to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-mpmc = "0.1"
```

Next, you can use `tokio-mpmc` as shown in the following example:

```rust
use tokio_mpmc::Queue;
use tokio::time::{self, Duration};

#[tokio::main]
async fn main() {
    // Create a queue with capacity 100
    let queue = Queue::new(100);

    // Clone the queue for multiple producers and consumers
    let producer_queue = queue.clone();
    let consumer_queue = queue.clone();

    // Start a producer task
    let producer_handle = tokio::spawn(async move {
        for i in 0..10 {
            let msg = format!("message {}", i);
            println!("Producer sending: {}", msg);
            if let Err(e) = producer_queue.send(msg).await {
                eprintln!("Producer send failed: {}", e);
                break;
            }
            time::sleep(Duration::from_millis(10)).await;
        }
        println!("Producer finished.");
    });

    // Start a consumer task
    let consumer_handle = tokio::spawn(async move {
        loop {
            match consumer_queue.receive().await {
                Ok(Some(msg)) => {
                    println!("Consumer received: {}", msg);
                } 
                Ok(None) => {
                    // Queue closed and empty
                    println!("Consumer finished: Queue closed and empty.");
                    break;
                }
                Err(e) => {
                    eprintln!("Consumer receive failed: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for producer and consumer tasks to finish
    producer_handle.await.unwrap();

    // Close the queue to notify consumers no more messages
    println!("Closing queue...");
    queue.close().await;

    // Wait for consumer task to finish
    consumer_handle.await.unwrap();

    println!("Example finished.");
}
```

## Brief Internal Implementation

The core implementation of `tokio-mpmc` revolves around the following key components:

1. **`Queue<T>` struct**: The main interface provided to users. It's a clonable handle that internally shares the actual queue state via `Arc<Inner<T>>`, supporting multiple producers and consumers.

2. **`Inner<T>` struct**: Contains the actual queue state and synchronization primitives. It's the internal detail of `Queue<T>`, shared among handles via `Arc`.

3. **`crossbeam_queue::ArrayQueue<T>`**: The `Inner` struct uses the `ArrayQueue` from the `crossbeam-queue` library as the underlying buffer. `ArrayQueue` is a bounded, lock-free MPMC queue, ideal for concurrent access in multithreaded or multitask environments with high performance.

4. **`std::sync::atomic` atomic types**: `Inner` uses `AtomicBool` (`is_closed`) and `AtomicUsize` (`count`) to safely share and modify the queue's closed state and current element count among tasks. Atomic operations ensure thread safety for these state updates, avoiding lock overhead.

5. **`tokio::sync::Notify`**: The `Inner` struct contains two `Notify` instances: `producer_waiters` and `consumer_waiters`. `Notify` is Tokio's async synchronization primitive for sending "notification" signals between tasks.
    - When a producer tries to send data to a full queue, it waits on `producer_waiters` (`notified().await`) until space is available.
    - When a consumer tries to receive from an empty queue, it waits on `consumer_waiters` (`notified().await`) until new data arrives.
    - When data is successfully sent or received, the corresponding `notify_one()` or `notify_waiters()` is called to wake up waiting tasks.

## Workflow

### Send

1. Producer calls `queue.send(value).await`.
2. Checks the queue's `is_closed` state. If closed, immediately returns `Err(QueueError::QueueClosed)`.
3. Attempts to push data into the underlying `ArrayQueue` with `self.inner.buffer.push(value)`.
4. If `push` succeeds, atomically increments `count`, notifies a waiting consumer via `consumer_waiters.notify_one()`, and returns `Ok(())`.
5. If `push` fails (queue full), checks `is_closed` again. If closed, returns `Err(QueueError::QueueClosed)`.
6. If not closed but full, the producer task suspends at `producer_waiters.notified().await`, waiting for a consumer to free up space.
7. Upon waking, loops back to step 3 to retry sending.

### Receive

1. Consumer calls `queue.receive().await`.
2. Attempts to pop data from the underlying `ArrayQueue` with `self.inner.buffer.pop()`.
3. If `pop` succeeds, atomically decrements `count`, notifies a waiting producer via `producer_waiters.notify_one()`, and returns `Ok(Some(value))`.
4. If `pop` fails (queue empty), checks `is_closed`.
5. If closed and `count` is 0 (queue completely empty), returns `Ok(None)`.
6. If closed but `count` is not 0 (shouldn't happen in theory, but handled as an error), returns `Err(QueueError::QueueClosed)`.
7. If not closed but empty, the consumer task suspends at `consumer_waiters.notified().await`, waiting for a producer to add new data.
8. Upon waking, loops back to step 2 to retry receiving.

### Close

1. Calls `queue.close().await`.
2. Atomically sets the `is_closed` flag to `true`.
3. Calls `producer_waiters.notify_waiters()` and `consumer_waiters.notify_waiters()` to wake all waiting producer and consumer tasks.
4. Woken tasks check the `is_closed` flag and return the appropriate error or `Ok(None)` according to the send/receive logic.

## Capacity Control & Backpressure

`tokio-mpmc` uses a fixed-capacity `ArrayQueue`, naturally supporting bounded queues. When the queue reaches its capacity limit, subsequent `send` operations cause producer tasks to suspend until space is available. This provides a natural backpressure mechanism, preventing producers from overwhelming memory.

## Error Handling

The library defines a `QueueError` enum to represent possible errors, currently mainly including `QueueClosed` to indicate the queue was closed during operations.

## Conclusion

`tokio-mpmc` provides a powerful and flexible MPMC queue solution for Tokio-based async applications. Whether building high-performance network services, handling concurrent tasks, or enabling async communication between components, `tokio-mpmc` offers reliable support. By leveraging its async features and simple API, developers can more easily build efficient and scalable concurrent applications.

We hope this technical article helps you understand `tokio-mpmc` and start using it in your projects!