#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_basic_operations() {
        let (tx, rx) = tokio_mpmc::channel(2);

        // Test sending
        assert!(tx.send(1).await.is_ok());
        assert!(tx.send(2).await.is_ok());
        // Test receiving
        // Add checks before receiving
        assert_eq!(tx.len(), 2);
        assert!(!tx.is_empty());
        assert!(tx.is_full());

        if let Ok(Some(value)) = rx.recv().await {
            assert_eq!(value, 1);
        } else {
            panic!("Failed to receive value 1"); // Added panic for clarity if receive fails
        }

        // Add checks after first receive
        assert_eq!(rx.len(), 1);
        assert!(!rx.is_empty());
        assert!(!rx.is_full());

        if let Ok(Some(value)) = rx.recv().await {
            assert_eq!(value, 2);
        } else {
            panic!("Failed to receive value 2"); // Added panic for clarity if receive fails
        }

        assert!(rx.is_empty());
        assert_eq!(rx.len(), 0); // Added len check
        assert!(!rx.is_full()); // Added full check
    }

    #[tokio::test]
    async fn test_close() {
        let (tx, rx) = tokio_mpmc::channel(1);
        tx.close();

        assert!(tx.send(1).await.is_err());
        assert_eq!(rx.recv().await.unwrap(), None);
        assert!(tx.is_closed());
    }

    #[tokio::test]
    async fn test_producer_waits_when_full() {
        let (tx, rx) = tokio_mpmc::channel(1);
        assert!(tx.send(1).await.is_ok());

        // This send should block because the channel is full
        let product_tx = tx.clone();
        let producer_task = tokio::spawn(async move { product_tx.send(2).await });

        // Give the producer task a moment to potentially block
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // The channel should still be full, and the producer task should not have finished yet
        assert!(tx.is_full());
        assert!(!producer_task.is_finished());

        // Receive an item, which should unblock the producer
        assert_eq!(rx.recv().await.unwrap(), Some(1));

        // The producer task should now complete successfully
        let producer_result = producer_task.await.unwrap();
        assert!(producer_result.is_ok());
        assert_eq!(tx.len(), 1);

        // Receive the item sent by the producer task
        assert_eq!(rx.recv().await.unwrap(), Some(2));
        assert!(rx.is_empty());
    }

    #[tokio::test]
    async fn test_consumer_waits_when_empty() {
        let (tx, rx) = tokio_mpmc::channel(1);

        // This receive should block because the channel is empty
        let consumer_rx = rx.clone();
        let consumer_task = tokio::spawn(async move { consumer_rx.recv().await });

        // Give the consumer task a moment to potentially block
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // The consumer task should not have finished yet
        assert!(!consumer_task.is_finished());

        // Send an item, which should unblock the consumer
        assert!(tx.send(1).await.is_ok());

        // The consumer task should now complete successfully and receive the item
        let consumer_result = consumer_task.await.unwrap().unwrap();
        assert_eq!(consumer_result, Some(1));
        assert!(rx.is_empty());
    }

    #[tokio::test]
    async fn test_close_unblocks_waiters() {
        let (tx, rx) = tokio_mpmc::channel(1);

        // Producer task that will block
        let channel_clone_producer = tx.clone();
        let producer_task = tokio::spawn(async move {
            let _ = channel_clone_producer.send(1).await; // This will succeed
            let _ = channel_clone_producer.send(2).await; // This will block
        });

        // Consumer task that will block
        let channel_clone_consumer = rx.clone();
        let consumer_task = tokio::spawn(async move {
            let _ = channel_clone_consumer.recv().await; // This will succeed after producer sends 1
            let _ = channel_clone_consumer.recv().await; // This will block
        });

        // Give tasks a moment to reach blocking state
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Close the channel
        tx.close();

        // Wait for tasks to finish (they should be unblocked by close)
        producer_task.await.unwrap();
        consumer_task.await.unwrap();

        // Check that the blocking operations returned channelClosed error
        // The first send/receive in each task should succeed before blocking
        // The second send/receive should return channelClosed
        // Note: The exact return value for the second send/receive depends on timing,
        // but closing should cause them to unblock.
        // We can check the channel state after closing.
        assert!(tx.is_closed());
        assert!(tx.is_empty()); // All items should be consumed or dropped

        // More specific checks on task results might be needed depending on exact behavior
        // For simplicity, we just check they finished and the channel state is correct.
    }
}
