use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::{self, MissedTickBehavior};

use crate::request::Request;
use crate::scheduler::{Scheduler, SchedulerEvent};

pub async fn run(
    mut scheduler: Scheduler,
    mut requests: mpsc::Receiver<Request>,
    events: mpsc::Sender<SchedulerEvent>,
) -> Result<(), String> {
    let mut ticker = time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut receiving = true;

    while receiving || scheduler.has_work() {
        tokio::select! {
            request = requests.recv(), if receiving => {
                match request {
                    Some(request) => scheduler.add_request(request)?,
                    None => receiving = false,
                }
            }
            _ = ticker.tick(), if scheduler.has_work() => {
                for event in scheduler.step() {
                    events
                        .send(event)
                        .await
                        .map_err(|error| error.to_string())?;
                }
            }
        }
    }

    Ok(())
}
