use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::AcIContainer;

use std::cmp::Ordering;

/// Compares AcIContainer using the order they have been registered with the ADC.
/// Additionally, a start position can be given at creation time. Then AcIs
/// registered earlier will be assumed to be at the end of the list.
pub struct PositionCompare {
    start: usize,
}

impl PositionCompare {
    /// Create new comparator with a given start value offset.
    ///
    /// * `start_value`: all AcIs registered earlier (with a lower internal number) are assumed
    ///                  to be appended at the end of the list. But still in the order of the registration.
    ///
    ///  Note: In case both acis are the same an error is logged and `Ordering::Equal` is returned.
    pub fn new(start_value: usize) -> Self {
        Self { start: start_value }
    }

    pub fn compare(&self, aci1: &AcIContainer, aci2: &AcIContainer) -> Ordering {
        if aci1.registration_index == aci2.registration_index {
            return Ordering::Equal;
        }

        if (aci1.registration_index < self.start) && (aci2.registration_index >= self.start) {
            // ai1 is before the marker, but aci2 after -> ai1 has higher rank (comes later)
            Ordering::Greater
        } else if (aci1.registration_index >= self.start) && (aci2.registration_index < self.start) {
            // aci1 is after the marker, but aci2 is before -> aci2 has higher rank (comes later)
            Ordering::Less
        } else {
            // both are on the same side of the marker, just compare
            aci1.registration_index.cmp(&aci2.registration_index)
        }
    }
}
