mod request;
mod scheduler;

use request::Request;
use scheduler::Scheduler;

fn main() {
    let mut scheduler = Scheduler::new();

    scheduler.add_request(Request::new(1, 2));
    scheduler.add_request(Request::new(2, 4));
    scheduler.add_request(Request::new(3, 3));

    while scheduler.has_work() {
        scheduler.step();
    }
}
