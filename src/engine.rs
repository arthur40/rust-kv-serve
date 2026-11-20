use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio::time::{self, MissedTickBehavior};

use crate::request::Request;
use crate::scheduler::{Scheduler, SchedulerEvent};

pub enum EngineCommand {
    Submit(Request),
    Cancel {
        id: u64,
        reply: oneshot::Sender<bool>,
    },
}

pub async fn run(
    mut scheduler: Scheduler,
    mut commands: mpsc::Receiver<EngineCommand>,
    events: mpsc::Sender<SchedulerEvent>,
) -> Result<(), String> {
    let mut ticker = time::interval(Duration::from_millis(200));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut receiving = true;

    while receiving || scheduler.has_work() {
        tokio::select! {
            command = commands.recv(), if receiving => {
                match command {
                    Some(EngineCommand::Submit(request)) => {
                        scheduler.add_request(request)?;
                    }
                    Some(EngineCommand::Cancel { id, reply }) => {
                        let cancelled = scheduler.cancel(id);
                        let _ = reply.send(cancelled);
                    }
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

#[cfg(test)]
mod tests {
    use super::{EngineCommand, run};
    use crate::request::Request;
    use crate::scheduler::{Scheduler, SchedulerEvent};
    use tokio::sync::{mpsc, oneshot};
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn cancellation_frees_capacity_for_another_request() {
        timeout(Duration::from_secs(3), async {
            let scheduler = Scheduler::new(1, 100);
            let (command_tx, command_rx) = mpsc::channel(16);
            let (event_tx, mut event_rx) = mpsc::channel(32);

            let task = tokio::spawn(run(scheduler, command_rx, event_tx));

            command_tx
                .send(EngineCommand::Submit(Request::new(1, 100)))
                .await
                .unwrap();

            assert!(matches!(
                event_rx.recv().await,
                Some(SchedulerEvent::TokenGenerated { id: 1, .. })
            ));

            let (reply_tx, reply_rx) = oneshot::channel();

            command_tx
                .send(EngineCommand::Cancel {
                    id: 1,
                    reply: reply_tx,
                })
                .await
                .unwrap();

            assert!(reply_rx.await.unwrap());

            command_tx
                .send(EngineCommand::Submit(Request::new(2, 1)))
                .await
                .unwrap();

            drop(command_tx);

            let mut second_finished = false;

            while let Some(event) = event_rx.recv().await {
                match event {
                    SchedulerEvent::Finished { id: 1 } => {
                        panic!("Cancelled request finished normally");
                    }
                    SchedulerEvent::Finished { id: 2 } => {
                        second_finished = true;
                    }
                    _ => {}
                }
            }

            assert!(second_finished);
            task.await.unwrap().unwrap();
        })
        .await
        .expect("Engine did not finish");
    }
}
