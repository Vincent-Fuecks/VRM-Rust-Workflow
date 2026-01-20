use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_manager::VrmComponentContainer;
use crate::domain::vrm_system_model::grid_resource_management_system::vrm_component_trait::VrmComponent;

use std::cmp::Ordering;

/// Compares VrmComponentContainer by the load known to this ADC. It only takes load into
/// account submitted by this ADC unit, so the ordering may differ from the real load ordering.
pub struct LoadCompare {
    start: i64,
    end: i64,
}

impl LoadCompare {
    /// Create new comparator with a given time frame.
    ///
    /// * `start_value`: specifies the beginning of the time frame to use for the comparison.
    /// * `end_value`: specifies the end of the time frame, which is utilized for the comparison.
    pub fn new(start_value: i64, end_value: i64) -> Self {
        Self { start: start_value, end: end_value }
    }

    /// TODO Performance Bottleneck: get_load_metric is 2 * N * log(N) times called
    /// Returns `Ordering::Less`, if aci1 has a lower load than aci2
    ///         `Ordering::Greater`, if aci1 has a higher load than aci2
    ///
    /// Note: if load of aci1 and aci2 are equal, is the registration_index of both acis compared.
    ///       In case both acis are the same `Ordering::Equal` is returned.
    pub fn compare(&self, aci1: &VrmComponentContainer, aci2: &VrmComponentContainer) -> Ordering {
        let load1 = aci1.vrm_component.get_load_metric(self.start as i64, self.end as i64, None).utilization;
        let load2 = aci2.vrm_component.get_load_metric(self.start as i64, self.end as i64, None).utilization;

        if aci1.registration_index == aci2.registration_index {
            return Ordering::Equal;
        }

        match load1.partial_cmp(&load2) {
            Some(Ordering::Equal) | None => aci1.registration_index.cmp(&aci2.registration_index),
            Some(ord) => ord,
        }
    }
}
