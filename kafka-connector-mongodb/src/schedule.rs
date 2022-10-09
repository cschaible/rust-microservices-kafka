use std::sync::Arc;
use std::time::Duration;

use opentelemetry_propagator_b3::propagator::Propagator;
use rdkafka::producer::FutureProducer;
use tokio::sync::Mutex;
use tokio_cron_scheduler::Job;
use tokio_cron_scheduler::JobScheduler;

use crate::common::context::DynContext;
use crate::common::db::transactional2;
use crate::job;

pub async fn run_scheduled_job(
    context: DynContext,
    producer: Arc<FutureProducer>,
    tracing_propagator: Arc<Propagator>,
) {
    let job_synchronization_mutex = Arc::new(Mutex::new(false));

    let scheduler = JobScheduler::new()
        .await
        .expect("Job scheduler couldn't be instantiated");

    let job = Job::new_repeated_async(Duration::from_secs(1), move |_job_id, _lock| {
        let p_job_synchronization_mutex = job_synchronization_mutex.clone();
        let p_producer = producer.clone();
        let p_tracing_propagator = tracing_propagator.clone();
        let p_context = context.clone();

        Box::pin(async move {
            transactional2(p_context, |tx| {
                let p_inner_job_synchronization_mutex = p_job_synchronization_mutex.clone();
                let p_inner_producer = p_producer.clone();
                let p_inner_tracing_propagator = p_tracing_propagator.clone();
                Box::pin(async move {
                    job::poll_and_send(
                        p_inner_job_synchronization_mutex,
                        tx,
                        p_inner_producer.clone(),
                        p_inner_tracing_propagator.clone(),
                    )
                    .await;
                    Ok(())
                })
            })
            .await
            .expect("Sending events failed");
        })
    })
    .expect("Job couldn't be instantiated");
    scheduler.add(job).await.expect("Job couldn't be scheduled");

    #[cfg(feature = "signal")]
    scheduler.shutdown_on_ctrl_c();
    scheduler
        .start()
        .await
        .expect("Job scheduler couldn't be started");
}
