mod backend;
mod kv_cache;
mod request;
mod scheduler;

use request::Request;
use scheduler::{Scheduler, SchedulerEvent};

fn main() -> Result<(), String> {
    let mut scheduler = Scheduler::new(2, 5);

    scheduler.add_request(Request::new(1, 2))?;
    scheduler.add_request(Request::new(2, 4))?;
    scheduler.add_request(Request::new(3, 3))?;

    let mut round = 0;

    while scheduler.has_work() {
        round += 1;
        println!("Round {round}");

        for event in scheduler.step() {
            match event {
                SchedulerEvent::TokenGenerated { id, count, token } => {
                    println!("Request {id}: token {count} = {token}");
                }
                SchedulerEvent::Finished { id } => {
                    println!("Request {id} finished");
                }
            }
        }
    }

    Ok(())
}
