use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use tokio;
use tokio_mpmc::Queue;

pub async fn run_tokio_mpmc_io_test(
    queue_size: u32,
    num_producers: u32,
    num_consumers: u32,
) -> std::time::Duration {
    println!(
        "Starting tokio-mpmc io performance test with queue size: {}, producers: {}, consumers: {}",
        queue_size, num_producers, num_consumers
    );

    let queue = Queue::new(queue_size as usize);
    let start_time = Instant::now();

    // Spawn producer tasks
    let mut producer_handles = vec![];
    for i in 0..num_producers {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            let messages_per_producer = queue_size / num_producers;
            let start_message = i * messages_per_producer;
            let end_message = start_message + messages_per_producer;
            for msg in start_message..end_message {
                if queue_clone.send(msg).await.is_err() {
                    eprintln!("Producer {} failed to send message {}", i, msg);
                    break;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Spawn consumer tasks
    let mut consumer_handles = vec![];
    for i in 0..num_consumers {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            let messages_per_consumer = queue_size / num_consumers;
            let mut received_count = 0;
            while received_count < messages_per_consumer {
                match queue_clone.receive().await {
                    Ok(Some(msg)) => {
                        received_count += 1;
                        let mut file = OpenOptions::new()
                            .append(true)
                            .create(true)
                            .open("output.txt")
                            .expect("Failed to open file");
                        writeln!(file, "Received message: {}", msg)
                            .expect("Failed to write to file");
                    }
                    Ok(None) => {
                        // Queue closed and empty
                        break;
                    }
                    Err(e) => {
                        eprintln!("Consumer {} failed to receive message: {:?}", i, e);
                        break;
                    }
                }
            }
        });
        consumer_handles.push(handle);
    }

    // Wait for all producers to finish
    for handle in producer_handles {
        let _ = handle.await;
    }

    // Wait for all consumers to finish
    for handle in consumer_handles {
        let _ = handle.await;
    }

    let duration = start_time.elapsed();

    let _ = tokio::fs::remove_file("output.txt").await;
    println!("tokio-mpmc io performance test finished in: {:?}", duration);

    // Close the queue after all producers are done
    queue.close().await;

    duration
}
