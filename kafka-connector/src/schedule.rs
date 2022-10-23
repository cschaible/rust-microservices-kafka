use std::sync::Arc;
use std::time::Duration;

use common_error::AppError;
use opentelemetry_propagator_b3::propagator::Propagator;
use rdkafka::producer::FutureProducer;
use sea_orm::DatabaseConnection;
use tokio::sync::Mutex;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

use crate::job;

pub async fn run_scheduled_job(
    connection: Arc<DatabaseConnection>,
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
) -> Result<(), AppError> {
    let job_synchronization_mutex = Arc::new(Mutex::new(false));

    let job = Job::new_repeated_async(Duration::from_secs(1), move |_job_id, _lock| {
        let p_job_synchronization_mutex = job_synchronization_mutex.clone();
        let p_connection = connection.clone();
        let p_producer = producer.clone();
        let p_tracing_propagator = tracing_propagator.clone();

        Box::pin(async move {
            job::poll_and_send(
                p_job_synchronization_mutex,
                p_connection.clone(),
                p_producer.clone(),
                p_tracing_propagator.clone(),
            )
            .await
            .expect("Scheduled job failed")
        })
    })?;

    let scheduler = JobScheduler::new().await?;
    scheduler.add(job).await?;

    #[cfg(feature = "signal")]
    scheduler.shutdown_on_ctrl_c();

    scheduler.start().await?;

    Ok(())
}
