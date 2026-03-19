use parking_lot::Mutex;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

pub struct IpRateLimiter {
    buckets: Arc<Mutex<HashMap<IpAddr, TokenBucket>>>,
    max_requests: u32,
    window_seconds: f64,
}

struct TokenBucket {
    tokens: f64,
    last_refill: std::time::Instant,
}

impl IpRateLimiter {
    pub fn new(max_requests: u32, window_seconds: f64) -> Self {
        Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_seconds,
        }
    }

    pub fn check(&self, ip: IpAddr) -> bool {
        let mut buckets = self.buckets.lock();
        let now = std::time::Instant::now();

        let bucket = buckets.entry(ip).or_insert_with(|| TokenBucket {
            tokens: f64::from(self.max_requests),
            last_refill: now,
        });

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        let refill_rate = f64::from(self.max_requests) / self.window_seconds;
        bucket.tokens = (bucket.tokens + elapsed * refill_rate).min(f64::from(self.max_requests));
        bucket.last_refill = now;

        // Check if we have tokens available
        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_rate_limiter_allows_requests_within_quota() {
        // Given
        let limiter = IpRateLimiter::new(5, 1.0);
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        // When & Then
        for _ in 0..5 {
            assert!(limiter.check(ip), "Request should be allowed within quota");
        }
    }

    #[test]
    fn test_rate_limiter_blocks_excess_requests() {
        // Given
        let limiter = IpRateLimiter::new(2, 1.0);
        let ip = IpAddr::from_str("127.0.0.1").unwrap();

        // When & Then
        assert!(limiter.check(ip));
        assert!(limiter.check(ip));

        assert!(
            !limiter.check(ip),
            "Request should be blocked when quota exceeded"
        );
    }

    #[test]
    fn test_rate_limiter_per_ip() {
        // Given
        let limiter = IpRateLimiter::new(1, 1.0);
        let ip1 = IpAddr::from_str("127.0.0.1").unwrap();
        let ip2 = IpAddr::from_str("127.0.0.2").unwrap();

        // When & Then
        assert!(limiter.check(ip1));
        assert!(limiter.check(ip2));
        assert!(!limiter.check(ip1), "IP1 should be rate limited");
        assert!(!limiter.check(ip2), "IP2 should be rate limited");
    }
}
