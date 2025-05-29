use benchmark::tokio_mpsc_benchmark::TokioMpscChannel;
use criterion::{Criterion, criterion_group, criterion_main};
use tokio_test::block_on;

mod benchmark;

use benchmark::BenchmarkConfig;
use benchmark::flume_benchmark::FlumeQueue;
use benchmark::tokio_mpmc_channel_benchmark::TokioMpmcChannel;
use benchmark::tokio_mpmc_queue_benchmark::TokioMpmcQueue;

fn bench_queues(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue-comparison");
    let config = BenchmarkConfig::default();

    // Configure benchmark group
    group.sample_size(config.sample_size);
    group.measurement_time(config.measurement_time);
    group.warm_up_time(config.warm_up_time);

    // tokio-mpsc channel non-IO test
    group.bench_function("tokio-mpsc-channel/non-io", |b| {
        b.iter(|| {
            block_on(benchmark::run_non_io_mpsc_channel_benchmark::<
                TokioMpscChannel,
            >(&config))
        });
    });

    // tokio-mpsc channel IO test
    group.bench_function("tokio-mpsc-channel/io", |b| {
        b.iter(|| block_on(benchmark::run_io_mpsc_channel_benchmark::<TokioMpscChannel>(&config)));
    });

    // tokio-mpmc channel non-IO test
    group.bench_function("tokio-mpmc-channel/non-io", |b| {
        b.iter(|| {
            block_on(benchmark::run_non_io_channel_benchmark::<TokioMpmcChannel>(
                &config,
            ))
        });
    });

    // tokio-mpmc channel IO test
    group.bench_function("tokio-mpmc-channel/io", |b| {
        b.iter(|| {
            block_on(benchmark::run_io_channel_benchmark::<TokioMpmcChannel>(
                &config,
            ))
        });
    });

    // tokio-mpmc queue non-IO test
    group.bench_function("tokio-mpmc-queue/non-io", |b| {
        b.iter(|| block_on(benchmark::run_non_io_benchmark::<TokioMpmcQueue>(&config)));
    });

    // tokio-mpmc queue IO test
    group.bench_function("tokio-mpmc-queue/io", |b| {
        b.iter(|| block_on(benchmark::run_io_benchmark::<TokioMpmcQueue>(&config)));
    });

    // flume non-IO test
    group.bench_function("flume/non-io", |b| {
        b.iter(|| block_on(benchmark::run_non_io_benchmark::<FlumeQueue>(&config)));
    });

    // flume IO test
    group.bench_function("flume/io", |b| {
        b.iter(|| block_on(benchmark::run_io_benchmark::<FlumeQueue>(&config)));
    });

    group.finish();
}

criterion_group!(benches, bench_queues);
criterion_main!(benches);
