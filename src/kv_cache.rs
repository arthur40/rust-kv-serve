pub struct KvCache {
    capacity: usize,
    reserved: usize,
}

impl KvCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            reserved: 0,
        }
    }

    pub fn available(&self) -> usize {
        self.capacity - self.reserved
    }

    pub fn try_reserve(&mut self, tokens: usize) -> bool {
        if tokens > self.available() {
            return false;
        }

        self.reserved += tokens;
        true
    }

    pub fn release(&mut self, tokens: usize) {
        assert!(tokens <= self.reserved);
        self.reserved -= tokens;
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::KvCache;

    #[test]
    fn reservations_cannot_exceed_capacity() {
        let mut cache = KvCache::new(10);

        assert!(cache.try_reserve(6));
        assert_eq!(cache.available(), 4);

        assert!(!cache.try_reserve(5));
        assert_eq!(cache.available(), 4);

        assert!(cache.try_reserve(4));
        assert_eq!(cache.available(), 0);
    }

    #[test]
    fn released_capacity_can_be_reused() {
        let mut cache = KvCache::new(10);

        assert!(cache.try_reserve(10));

        cache.release(6);

        assert_eq!(cache.available(), 6);
        assert!(cache.try_reserve(6));
    }

    #[test]
    #[should_panic]
    fn releasing_more_than_reserved_is_rejected() {
        let mut cache = KvCache::new(10);

        assert!(cache.try_reserve(3));
        cache.release(4);
    }
}
