use std::time::Instant;

pub struct RateLimiter {
    tokens: f64,
    last_update: Instant,
    capacity: f64,
    refill_rate_ms: f64,
}

impl RateLimiter {
    pub fn new(capacity: f64, refill_rate_ms: f64) -> Self {
        Self {
            tokens: capacity,
            last_update: Instant::now(),
            capacity,
            refill_rate_ms,
        }
    }

    pub fn take(&mut self) -> bool {
        let now = Instant::now();
        let elapsed_ms = now.duration_since(self.last_update).as_secs_f64() * 1000.0;
        
        // Refill tokens based on elapsed time
        if self.refill_rate_ms > 0.0 {
            let added_tokens = elapsed_ms / self.refill_rate_ms;
            self.tokens += added_tokens;
            if self.tokens > self.capacity {
                self.tokens = self.capacity;
            }
            self.last_update = now;
        }

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}
