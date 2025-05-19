mod tests;

use tests::tokio_mpmc_io_test::run_tokio_mpmc_io_test;
use tests::tokio_mpmc_test::run_tokio_mpmc_test;
use tests::tokio_mpsc_io_test::run_tokio_mpsc_io_test;
use tests::tokio_mpsc_test::run_tokio_mpsc_test;

#[tokio::main]
async fn main() {
    let queue_size = 1_000_000;
    let num_producers = 4;
    let num_consumers = 4;

    run_tokio_mpmc_test(
        queue_size as u32,
        num_producers as u32,
        num_consumers as u32,
    )
    .await;
    run_tokio_mpsc_test(queue_size as u32, num_producers as u32).await;

    let queue_size = 100_000;
    run_tokio_mpmc_io_test(
        queue_size as u32,
        num_producers as u32,
        num_consumers as u32,
    )
    .await;
    run_tokio_mpsc_io_test(queue_size as u32, num_producers as u32).await;
}
