use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();

    let (tx, rx) = tokio_mpmc::channel(100);

    let db_handle = tokio::spawn(async move {
        let mut i = 0;
        loop {
            match rx.recv().await {
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

    send_task(tx).await;

    db_handle.await.unwrap();
}

async fn send_task(tx: tokio_mpmc::Sender<i32>) {
    let mut tasks = JoinSet::new();

    for i in 0..10 {
        let tx = tx.clone();
        tasks.spawn(async move {
            let _ = tx.send(i).await;
        });
    }

    while let Some(result) = tasks.join_next().await {
        result.unwrap();
    }
}
