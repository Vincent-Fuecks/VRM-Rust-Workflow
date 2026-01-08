use crate::domain::vrm_system_model::reservation::reservation_store::ReservationStore;

/**
 * A workflow scheduler is responsible to handle all workflows in the
 * {@link ADC}. This class is the base class for various workflow
 * schedule implementations.
 *
 * This class provides methods for committing and deleting workflows, which may be
 * overwritten or used by the subclass. Additionally, the subclasses have to provide
 * means for probing and reserving reservations.
 *
 * By convention, all subclasses have to provide a constructor like
 * {@link #WorkflowScheduler(ADCcore)}.
 *
 * @see ADC
 * @see Workflow
 */

pub struct WorkflowSchedulerBase {
    pub reservation_store: ReservationStore,
}
