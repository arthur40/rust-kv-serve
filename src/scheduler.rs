use std::collections::VecDeque;

use crate::request::Request;

pub struct Scheduler {
    active: Vec<Request>,
    waiting: VecDeque<Request>,
    max_active: usize,
}

impl Scheduler {
    pub fn new(max_active: usize) -> Self {
        assert!(max_active > 0);

        Self {
            active: Vec::new(),
            waiting: VecDeque::new(),
            max_active,
        }
    }

    pub fn add_request(&mut self, request: Request) {
        if !request.is_finished() {
            self.waiting.push_back(request);
        }
    }

    pub fn has_work(&self) -> bool {
        !self.active.is_empty() || !self.waiting.is_empty()
    }

    pub fn step(&mut self) {
        while self.active.len() < self.max_active {
            match self.waiting.pop_front() {
                Some(request) => self.active.push(request),
                None => break,
            }
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
        let mut scheduler = Scheduler::new(1);

        scheduler.add_request(Request::new(1, 2));
        scheduler.add_request(Request::new(2, 3));

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
        let mut scheduler = Scheduler::new(2);

        for id in 1..=5 {
            scheduler.add_request(Request::new(id, 3));
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
        let mut scheduler = Scheduler::new(1);

        scheduler.add_request(Request::new(1, 0));

        assert!(!scheduler.has_work());
    }

    #[test]
    #[should_panic]
    fn zero_active_limit_is_rejected() {
        Scheduler::new(0);
    }
}
