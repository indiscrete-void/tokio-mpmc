use criterion::{Criterion, criterion_group, criterion_main};
use tokio_test::block_on;

mod benchmark;

use benchmark::BenchmarkConfig;
use benchmark::flume_benchmark::FlumeQueue;
use benchmark::tokio_mpmc_benchmark::TokioMpmcQueue;
// use benchmark::tokio_mpsc_benchmark::TokioMpscQueue;

fn bench_queues(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue-comparison");
    let config = BenchmarkConfig::default();

    // Configure benchmark group
    group.sample_size(config.sample_size);
    group.measurement_time(config.measurement_time);
    group.warm_up_time(config.warm_up_time);

    // // tokio-mpsc non-IO test
    // let mpsc_config = BenchmarkConfig {
    //     num_consumers: 1, // mpsc only supports a single consumer
    //     ..config
    // };
    // group.bench_function("tokio-mpsc/non-io", |b| {
    //     b.iter(|| {
    //         block_on(benchmark::run_non_io_benchmark::<TokioMpscQueue>(
    //             &mpsc_config,
    //         ))
    //     });
    // });

    // // tokio-mpsc IO test
    // group.bench_function("tokio-mpsc/io", |b| {
    //     b.iter(|| block_on(benchmark::run_io_benchmark::<TokioMpscQueue>(&mpsc_config)));
    // });

    // tokio-mpmc non-IO test
    group.bench_function("tokio-mpmc/non-io", |b| {
        b.iter(|| block_on(benchmark::run_non_io_benchmark::<TokioMpmcQueue>(&config)));
    });

    // tokio-mpmc IO test
    group.bench_function("tokio-mpmc/io", |b| {
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
