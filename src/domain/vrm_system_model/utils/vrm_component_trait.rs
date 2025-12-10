use crate::domain::vrm_system_model::utils::statistics::StatisticEvent;

pub trait VRMComponent {
    /// Generates a comprehensive `StatisticEvent` containing key performance indicators
    /// for the component's current operational state.
    fn generate_statistics(&mut self) -> StatisticEvent;
}
