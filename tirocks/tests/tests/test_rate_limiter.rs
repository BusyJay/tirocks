// Copyright 2022 TiKV Project Authors. Licensed under Apache-2.0.

use std::time::Duration;

use tirocks::{
    env::IoPriority,
    rate_limiter::{OpType, RateLimiterBuilder},
};

#[test]
fn test_rate_limiter() {
    let rate_limiter = RateLimiterBuilder::new(10 * 1024 * 1024)
        .set_refill_period(Duration::from_millis(100))
        .set_fairness(10)
        .build_generic();
    assert_eq!(rate_limiter.single_burst_bytes(), 1 * 1024 * 1024);

    rate_limiter.set_bytes_per_second(20 * 1024 * 1024);
    assert_eq!(rate_limiter.bytes_per_second(), 20 * 1024 * 1024);

    assert_eq!(rate_limiter.single_burst_bytes(), 2 * 1024 * 1024);

    assert_eq!(rate_limiter.total_bytes_through(IoPriority::IO_TOTAL), 0);

    rate_limiter.request(1024 * 1024, IoPriority::IO_LOW, OpType::kWrite);
    assert_eq!(
        rate_limiter.total_bytes_through(IoPriority::IO_LOW),
        1024 * 1024
    );

    rate_limiter.request(2048 * 1024, IoPriority::IO_HIGH, OpType::kWrite);
    assert_eq!(
        rate_limiter.total_bytes_through(IoPriority::IO_HIGH),
        2048 * 1024
    );

    assert_eq!(
        rate_limiter.total_bytes_through(IoPriority::IO_TOTAL),
        3072 * 1024
    );
}

#[test]
fn test_rate_limiter_sendable() {
    let rate_limiter = RateLimiterBuilder::new(10 * 1024 * 1024)
        .set_refill_period(Duration::from_millis(100))
        .set_fairness(10)
        .build_generic();

    let handle = std::thread::spawn(move || {
        rate_limiter.request(1024, IoPriority::IO_LOW, OpType::kRead);
    });

    handle.join().unwrap();
}
