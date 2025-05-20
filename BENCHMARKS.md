# Benchmarks

## Table of Contents

- [Benchmark Results](#benchmark-results)
    - [queue-comparison](#queue-comparison)

## Benchmark Results

### queue-comparison

|              | `tokio-mpmc`              | `flume`                           |
|:-------------|:--------------------------|:--------------------------------- |
| **`non-io`** | `649.09 us` (✅ **1.00x**) | `768.68 us` (❌ *1.18x slower*)    |
| **`io`**     | `191.51 ms` (✅ **1.00x**) | `215.82 ms` (❌ *1.13x slower*)    |

---
Made with [criterion-table](https://github.com/nu11ptr/criterion-table)

