use std::collections::VecDeque;

use crate::kv_cache::KvCache;
use crate::request::Request;

pub struct Scheduler {
    active: Vec<Request>,
    waiting: VecDeque<Request>,
    max_active: usize,
    cache: KvCache,
}

impl Scheduler {
    pub fn new(max_active: usize, cache_capacity: usize) -> Self {
        assert!(max_active > 0);

        Self {
            active: Vec::new(),
            waiting: VecDeque::new(),
            max_active,
            cache: KvCache::new(cache_capacity),
        }
    }

    pub fn add_request(&mut self, request: Request) -> Result<(), String> {
        if request.max_tokens() > self.cache.capacity() {
            return Err(format!(
                "Request {} needs {} cache slots, but capacity is {}",
                request.id,
                request.max_tokens(),
                self.cache.capacity()
            ));
        }

        if !request.is_finished() {
            self.waiting.push_back(request);
        }

        Ok(())
    }

    pub fn has_work(&self) -> bool {
        !self.active.is_empty() || !self.waiting.is_empty()
    }

    pub fn step(&mut self) {
        while self.active.len() < self.max_active {
            let Some(request) = self.waiting.front() else {
                break;
            };

            if !self.cache.try_reserve(request.max_tokens()) {
                break;
            }

            let request = self.waiting.pop_front().unwrap();
            self.active.push(request);
        }

        for request in &mut self.active {
            request.step();

            println!(
                "Request {}: token {}",
                request.id,
                request.generated_tokens()
            );

            if request.is_finished() {
                println!("Request {} finished", request.id);
                self.cache.release(request.max_tokens());
            }
        }

        self.active.retain(|request| !request.is_finished());
    }
}

#[cfg(test)]
mod tests {
    use super::Scheduler;
    use crate::request::Request;

    #[test]
    fn waiting_request_starts_after_slot_opens() {
        let mut scheduler = Scheduler::new(1, 100);

        scheduler.add_request(Request::new(1, 2)).unwrap();
        scheduler.add_request(Request::new(2, 3)).unwrap();

        scheduler.step();

        assert_eq!(scheduler.active.len(), 1);
        assert_eq!(scheduler.active[0].id, 1);
        assert_eq!(scheduler.active[0].generated_tokens(), 1);
        assert_eq!(scheduler.waiting.len(), 1);
        assert_eq!(scheduler.waiting[0].id, 2);
        assert_eq!(scheduler.waiting[0].generated_tokens(), 0);

        scheduler.step();

        assert!(scheduler.active.is_empty());
        assert_eq!(scheduler.waiting.len(), 1);

        scheduler.step();

        assert_eq!(scheduler.active[0].id, 2);
        assert_eq!(scheduler.active[0].generated_tokens(), 1);
        assert!(scheduler.waiting.is_empty());

        scheduler.step();
        scheduler.step();

        assert!(!scheduler.has_work());
    }

    #[test]
    fn active_requests_stay_within_limit() {
        let mut scheduler = Scheduler::new(2, 100);

        for id in 1..=5 {
            scheduler.add_request(Request::new(id, 3)).unwrap();
        }

        scheduler.step();

        assert_eq!(scheduler.active.len(), 2);
        assert_eq!(scheduler.waiting.len(), 3);

        while scheduler.has_work() {
            scheduler.step();
            assert!(scheduler.active.len() <= 2);
        }
    }

    #[test]
    fn finished_requests_are_not_queued() {
        let mut scheduler = Scheduler::new(1, 100);

        scheduler.add_request(Request::new(1, 0)).unwrap();

        assert!(!scheduler.has_work());
    }

    #[test]
    #[should_panic]
    fn zero_active_limit_is_rejected() {
        Scheduler::new(0, 100);
    }

    #[test]
    fn cache_budget_blocks_admission_until_request_finishes() {
        let mut scheduler = Scheduler::new(2, 5);

        scheduler.add_request(Request::new(1, 2)).unwrap();
        scheduler.add_request(Request::new(2, 4)).unwrap();

        scheduler.step();

        assert_eq!(scheduler.active.len(), 1);
        assert_eq!(scheduler.active[0].id, 1);
        assert_eq!(scheduler.waiting.len(), 1);
        assert_eq!(scheduler.cache.available(), 3);

        scheduler.step();

        assert!(scheduler.active.is_empty());
        assert_eq!(scheduler.cache.available(), 5);

        scheduler.step();

        assert_eq!(scheduler.active.len(), 1);
        assert_eq!(scheduler.active[0].id, 2);
        assert!(scheduler.waiting.is_empty());
        assert_eq!(scheduler.cache.available(), 1);

        for _ in 0..3 {
            scheduler.step();
        }

        assert!(!scheduler.has_work());
        assert_eq!(scheduler.cache.available(), 5);
    }

    #[test]
    fn oversized_request_is_rejected_without_being_queued() {
        let mut scheduler = Scheduler::new(2, 5);

        let result = scheduler.add_request(Request::new(1, 6));

        assert!(result.is_err());
        assert!(!scheduler.has_work());
        assert_eq!(scheduler.cache.available(), 5);
    }

    #[test]
    fn requests_can_reserve_the_exact_cache_capacity() {
        let mut scheduler = Scheduler::new(2, 5);

        scheduler.add_request(Request::new(1, 2)).unwrap();
        scheduler.add_request(Request::new(2, 3)).unwrap();

        scheduler.step();

        assert_eq!(scheduler.active.len(), 2);
        assert!(scheduler.waiting.is_empty());
        assert_eq!(scheduler.cache.available(), 0);

        scheduler.step();

        assert_eq!(scheduler.active.len(), 1);
        assert_eq!(scheduler.cache.available(), 2);

        scheduler.step();

        assert!(!scheduler.has_work());
        assert_eq!(scheduler.cache.available(), 5);
    }
}
