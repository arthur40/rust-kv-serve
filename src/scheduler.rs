use crate::request::Request;

pub struct Scheduler {
    active: Vec<Request>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self { active: Vec::new() }
    }

    pub fn add_request(&mut self, request: Request) {
        if !request.is_finished() {
            self.active.push(request);
        }
    }

    pub fn has_work(&self) -> bool {
        !self.active.is_empty()
    }

    pub fn step(&mut self) {
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
