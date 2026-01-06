use crate::domain::vrm_system_model::grid_resource_management_system::aci_manager::AcIContainer;

use std::cmp::Ordering;

/// Compares AcIContainer by the size of the resources managed by the AcI
pub struct SizeCompare;

impl SizeCompare {
    pub fn new() -> Self {
        Self
    }

    /// Compares the two provided AcIs by teh size of the resources.
    ///
    /// Returns `Ordering::Less`, if aci1 has a lower total resource capacity (size) than aci2
    ///         `Ordering::Greater`, if aci1 has a higher total resource capacity (size) load than aci2
    ///
    /// Note: if resource capacity of aci1 and aci2 are equal, is the registration_index of both acis compared.
    ///       In case both acis are the same `Ordering::Equal` is returned.
    pub fn compare(&self, aci1: &mut AcIContainer, aci2: &mut AcIContainer) -> Ordering {
        if aci1.registration_index == aci2.registration_index {
            return Ordering::Equal;
        }

        let capacity1 = aci1.aci.get_total_capacity();
        let capacity2 = aci2.aci.get_total_capacity();

        match capacity1.cmp(&capacity2) {
            Ordering::Equal => aci1.registration_index.cmp(&aci2.registration_index),
            other => other,
        }
    }
}
