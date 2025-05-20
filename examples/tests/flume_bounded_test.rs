use std::time::Instant;

pub async fn run_flume_bounded_test(queue_size: u32, num_producers: u32) -> std::time::Duration {
    let num_consumers = 1;
    println!(
        "Starting flume::bounded performance test with queue size: {}, producers: {}, consumers: {}",
        queue_size, num_producers, num_consumers
    );

    let (tx, rx) = flume::bounded(queue_size as usize);
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
                if tx_clone.send(msg).is_err() {
                    eprintln!("mpsc Producer {} failed to send message {}", i, msg);
                    break;
                }
            }
        });
        producer_handles.push(handle);
    }

    // Spawn consumer tasks
    let mut consumer_handles = vec![];
    for i in 0..num_consumers {
        let rx_clone = rx.clone();
        let handle = tokio::spawn(async move {
            let messages_per_consumer = queue_size / num_consumers;
            let mut received_count = 0;
            while received_count < messages_per_consumer {
                match rx_clone.recv() {
                    Ok(_) => {
                        received_count += 1;
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

    println!(
        "flume::bounded performance test finished in: {:?}",
        duration
    );

    // Drop the original sender to signal consumers that no more messages will be sent
    drop(tx);

    duration
}
