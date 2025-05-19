#[cfg(test)]
mod tests {

    use tokio_mpmc::Queue;

    #[tokio::test]
    async fn test_basic_operations() {
        let queue = Queue::new(2);

        // Test sending
        assert!(queue.send(1).await.is_ok());
        assert!(queue.send(2).await.is_ok());
        // Test receiving
        // Add checks before receiving
        assert_eq!(queue.len(), 2);
        assert!(!queue.is_empty());
        assert!(queue.is_full());

        if let Ok(Some(value)) = queue.receive().await {
            assert_eq!(value, 1);
        } else {
            panic!("Failed to receive value 1"); // Added panic for clarity if receive fails
        }

        // Add checks after first receive
        assert_eq!(queue.len(), 1);
        assert!(!queue.is_empty());
        assert!(!queue.is_full());

        if let Ok(Some(value)) = queue.receive().await {
            assert_eq!(value, 2);
        } else {
            panic!("Failed to receive value 2"); // Added panic for clarity if receive fails
        }

        assert!(queue.is_empty());
        assert_eq!(queue.len(), 0); // Added len check
        assert!(!queue.is_full()); // Added full check
    }

    #[tokio::test]
    async fn test_close() {
        let queue = Queue::new(1);
        queue.close().await;

        assert!(queue.send(1).await.is_err());
        assert_eq!(queue.receive().await.unwrap(), None);
        assert!(queue.is_closed());
    }

    #[tokio::test]
    async fn test_producer_waits_when_full() {
        let queue = Queue::new(1);
        assert!(queue.send(1).await.is_ok());

        // This send should block because the queue is full
        let queue_clone = queue.clone();
        let producer_task = tokio::spawn(async move { queue_clone.send(2).await });

        // Give the producer task a moment to potentially block
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // The queue should still be full, and the producer task should not have finished yet
        assert!(queue.is_full());
        assert!(!producer_task.is_finished());

        // Receive an item, which should unblock the producer
        assert_eq!(queue.receive().await.unwrap(), Some(1));

        // The producer task should now complete successfully
        let producer_result = producer_task.await.unwrap();
        assert!(producer_result.is_ok());
        assert_eq!(queue.len(), 1);

        // Receive the item sent by the producer task
        assert_eq!(queue.receive().await.unwrap(), Some(2));
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_consumer_waits_when_empty() {
        let queue: Queue<i32> = Queue::new(1);

        // This receive should block because the queue is empty
        let queue_clone = queue.clone();
        let consumer_task = tokio::spawn(async move { queue_clone.receive().await });

        // Give the consumer task a moment to potentially block
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // The consumer task should not have finished yet
        assert!(!consumer_task.is_finished());

        // Send an item, which should unblock the consumer
        assert!(queue.send(1).await.is_ok());

        // The consumer task should now complete successfully and receive the item
        let consumer_result = consumer_task.await.unwrap().unwrap();
        assert_eq!(consumer_result, Some(1));
        assert!(queue.is_empty());
    }

    #[tokio::test]
    async fn test_close_unblocks_waiters() {
        let queue: Queue<i32> = Queue::new(1);

        // Producer task that will block
        let queue_clone_producer = queue.clone();
        let producer_task = tokio::spawn(async move {
            let _ = queue_clone_producer.send(1).await; // This will succeed
            let _ = queue_clone_producer.send(2).await; // This will block
        });

        // Consumer task that will block
        let queue_clone_consumer = queue.clone();
        let consumer_task = tokio::spawn(async move {
            let _ = queue_clone_consumer.receive().await; // This will succeed after producer sends 1
            let _ = queue_clone_consumer.receive().await; // This will block
        });

        // Give tasks a moment to reach blocking state
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Close the queue
        queue.close().await;

        // Wait for tasks to finish (they should be unblocked by close)
        producer_task.await.unwrap();
        consumer_task.await.unwrap();

        // Check that the blocking operations returned QueueClosed error
        // The first send/receive in each task should succeed before blocking
        // The second send/receive should return QueueClosed
        // Note: The exact return value for the second send/receive depends on timing,
        // but closing should cause them to unblock.
        // We can check the queue state after closing.
        assert!(queue.is_closed());
        assert!(queue.is_empty()); // All items should be consumed or dropped

        // More specific checks on task results might be needed depending on exact behavior
        // For simplicity, we just check they finished and the queue state is correct.
    }
}
