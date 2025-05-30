use tokio_mpmc::channel;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    // Create a channel with capacity 10
    let (tx, rx) = channel(10);

    // Spawn multiple receiver tasks
    let num_receivers = 3;
    let mut receiver_tasks = Vec::new();

    for i in 0..num_receivers {
        let rx = rx.clone();
        let task = tokio::spawn(async move {
            let mut count = 0;
            tracing::info!("Receiver {} started.", i);
            while let Ok(Some(value)) = rx.recv().await {
                tracing::info!(
                    "Receiver {} received value: {} capacity: {} max_capacity: {}",
                    i,
                    value,
                    rx.capacity(),
                    rx.max_capacity()
                );
                count += 1;
            }
            tracing::info!(
                "Receiver {} completed. Received count: {} capacity: {} max_capacity: {}",
                i,
                count,
                rx.capacity(),
                rx.max_capacity()
            );
        });
        receiver_tasks.push(task);
    }

    // Send values after receivers are ready
    for i in 0..100 {
        tx.send(i).await.unwrap();
    }

    // Wait for a while to let receivers process
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    // Close the channel after sending is complete
    drop(tx); // Drop the Sender to trigger the channel closure

    // Wait for all receiver tasks to complete
    for task in receiver_tasks {
        task.await.unwrap();
    }

    tracing::info!("All receivers completed.");
}
