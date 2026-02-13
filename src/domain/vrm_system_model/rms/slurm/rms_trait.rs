use std::fmt::Debug;

pub trait SlurmRestApi: Debug {
    // GET /slurm/v0.0.40/config
    fn init_rms(&self) -> bool;

    // GET /slurm/v0.0.40/nodes
    fn sync_nodes(&self) -> bool;

    // "http://localhost:6820/slurm/v0.0.40/jobs"
    fn sync_tasks(&self) -> bool;

    // Only Pending Jobs: /slurm/v0.0.40/jobs?state=PENDING
    fn get_waiting_task_for_execution(&self) -> bool;

    // POST /slurm/v0.0.40/node/{name}
    fn update_node_status(&self) -> bool;

    // GET /slurm/v0.0.40/ping
    fn is_rms_alive(&self) -> bool;

    // GET /slurm/v0.0.40/diag
    fn get_diagnostics(&self) -> bool;

    // POST /slurm/v0.0.40/job/submit
    fn commit(&self) -> bool;

    // DELETE /slurm/v0.0.40/job/{job_id}
    fn delete(&self) -> bool;
}
