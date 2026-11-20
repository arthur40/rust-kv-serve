mod backend;
mod kv_cache;
mod request;
mod scheduler;

use std::time::Duration;

use request::Request;
use scheduler::{Scheduler, SchedulerEvent};
use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut scheduler = Scheduler::new(2, 10);
    let (tx, mut rx) = mpsc::channel::<Request>(16);

    let producer = tokio::spawn(async move {
        for (id, max_tokens) in [(1, 5), (2, 4), (3, 3)] {
            tx.send(Request::new(id, max_tokens))
                .await
                .map_err(|error| error.to_string())?;

            time::sleep(Duration::from_millis(300)).await;
        }

        Ok::<(), String>(())
    });

    let mut ticker = time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut receiving = true;
    let mut round = 0;

    while receiving || scheduler.has_work() {
        tokio::select! {
            request = rx.recv(), if receiving => {
                match request {
                    Some(request) => {
                        println!("Received request {}", request.id);
                        scheduler.add_request(request)?;
                    }
                    None => receiving = false,
                }
            }
            _ = ticker.tick(), if scheduler.has_work() => {
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
        }
    }

    producer.await.map_err(|error| error.to_string())??;

    Ok(())
}
