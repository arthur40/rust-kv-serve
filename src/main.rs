mod backend;
mod engine;
mod kv_cache;
mod request;
mod scheduler;

use std::time::Duration;

use engine::EngineCommand;
use request::Request;
use scheduler::{Scheduler, SchedulerEvent};
use tokio::sync::mpsc;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), String> {
    let scheduler = Scheduler::new(2, 10);

    let (request_tx, request_rx) = mpsc::channel::<EngineCommand>(16);
    let (event_tx, mut event_rx) = mpsc::channel::<SchedulerEvent>(32);

    let engine_task = tokio::spawn(engine::run(scheduler, request_rx, event_tx));

    let producer = tokio::spawn(async move {
        for (id, max_tokens) in [(1, 5), (2, 4), (3, 3)] {
            request_tx
                .send(EngineCommand::Submit(Request::new(id, max_tokens)))
                .await
                .map_err(|error| error.to_string())?;

            time::sleep(Duration::from_millis(300)).await;
        }

        Ok::<(), String>(())
    });

    while let Some(event) = event_rx.recv().await {
        match event {
            SchedulerEvent::TokenGenerated { id, count, token } => {
                println!("Request {id}: token {count} = {token}");
            }
            SchedulerEvent::Finished { id } => {
                println!("Request {id} finished");
            }
        }
    }

    engine_task.await.map_err(|error| error.to_string())??;
    producer.await.map_err(|error| error.to_string())??;

    Ok(())
}
