mod tests;

use tests::flume_bounded_io_test::run_flume_bounded_io_test;
use tests::flume_bounded_test::run_flume_bounded_test;
use tests::tokio_mpmc_io_test::run_tokio_mpmc_io_test;
use tests::tokio_mpmc_test::run_tokio_mpmc_test;
use tests::tokio_mpsc_io_test::run_tokio_mpsc_io_test;
use tests::tokio_mpsc_test::run_tokio_mpsc_test;

#[tokio::main]
async fn main() {
    let queue_size = 1_000_000;
    let num_producers = 4;
    let num_consumers = 4;

    println!("============================================");
    println!("Starting non-IO tests");
    run_tokio_mpmc_test(
        queue_size as u32,
        num_producers as u32,
        num_consumers as u32,
    )
    .await;
    run_tokio_mpsc_test(queue_size as u32, num_producers as u32).await;
    run_flume_bounded_test(queue_size as u32, num_producers as u32).await;

    println!("============================================");
    println!("Starting IO tests");
    run_tokio_mpmc_io_test(
        queue_size as u32,
        num_producers as u32,
        num_consumers as u32,
    )
    .await;
    run_tokio_mpsc_io_test(queue_size as u32, num_producers as u32).await;
    run_flume_bounded_io_test(queue_size as u32, num_producers as u32).await;
}
