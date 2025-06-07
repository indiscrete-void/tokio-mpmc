use tokio::task::JoinSet;
use tokio_mpmc::Queue;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let queue = Queue::new(100);

    let rx_queue = queue.clone();
    let db_handle = tokio::spawn(async move {
        let mut i = 0;
        loop {
            match rx_queue.receive().await {
                Ok(Some(value)) => {
                    tracing::info!("DB received value: {}", value);
                    i += 1;
                }
                Ok(None) => {
                    tracing::info!("Channel closed and empty, exiting receiver");
                    break;
                }
                Err(e) => {
                    tracing::error!("Error receiving value: {:?}", e);
                    break;
                }
            }
        }
        tracing::info!("DB received {} values", i);
    });

    send_task(queue.clone()).await;

    db_handle.await.unwrap();
}

async fn send_task(queue: Queue<i32>) {
    let mut tasks = JoinSet::new();

    for i in 0..10 {
        let queue = queue.clone();
        tasks.spawn(async move {
            let _ = queue.send(i).await;
        });
    }

    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }

    drop(queue);
}
