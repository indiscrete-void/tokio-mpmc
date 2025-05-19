use std::fs::OpenOptions;
use std::io::Write;
use std::time::Instant;
use tokio;
use tokio::sync::mpsc;

pub async fn run_tokio_mpsc_io_test(queue_size: u32, num_producers: u32) -> std::time::Duration {
    let num_consumers = 1;
    println!(
        "Starting tokio::sync::mpsc performance test with queue size: {}, producers: {}, consumers: {}",
        queue_size, num_producers, num_consumers
    );

    let (tx, mut rx) = mpsc::channel(queue_size as usize);
    let start_time = Instant::now();

    // Spawn producer tasks
    let mut producer_handles = vec![];
    for i in 0..num_producers {
        let tx_clone = tx.clone();
        let handle = tokio::spawn(async move {
            let messages_per_producer = queue_size / num_producers;
            let start_message = i * messages_per_producer;
            let end_message = start_message + messages_per_producer;
            for msg in start_message..end_message {
                if tx_clone.send(msg).await.is_err() {
                    eprintln!("mpsc Producer {} failed to send message {}", i, msg);
                    break;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Spawn single consumer task
    let consumer_handle = tokio::spawn(async move {
        let mut received_count = 0;
        while received_count < queue_size {
            match rx.recv().await {
                Some(msg) => {
                    received_count += 1;
                    let mut file = OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open("output.txt")
                        .expect("Failed to open file");
                    writeln!(file, "Received message: {}", msg).expect("Failed to write to file");
                }
                None => {
                    // Channel closed and empty
                    break;
                }
            }
        }
    });

    // Wait for all producers to finish
    for handle in producer_handles {
        let _ = handle.await;
    }

    // Wait for consumer to finish
    let _ = consumer_handle.await;

    let duration = start_time.elapsed();

    let _ = tokio::fs::remove_file("output.txt").await;
    println!(
        "tokio::sync::mpsc performance test finished in: {:?}",
        duration
    );

    // Drop the original sender to signal consumers that no more messages will be sent
    drop(tx);

    duration
}
