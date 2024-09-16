use tokio_cron_scheduler::{JobSchedulerError, JobScheduler, Job};

use super::clear_containers::clear_containers;


pub async fn register_crons() -> Result<(), JobSchedulerError> {
    let job_scheduler = JobScheduler::new().await?;

    job_scheduler.add(
        Job::new_async("0 * * * * *", |_, _| Box::pin(clear_containers()))?
    ).await?;

    job_scheduler.start().await?;

    Ok(())
}
