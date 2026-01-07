use rand::rng;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::grid_resource_management_system::aci_order::AcIOrder;
use crate::domain::vrm_system_model::grid_resource_management_system::grid_resource_management_system_trait::ExtendedReservationProcessor;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId, ShadowScheduleId};
use crate::domain::vrm_system_model::utils::load_buffer::LoadMetric;

// TODO Functions must be synchronized with the AcIs
// TODO Old Java Version contained all resources and enabled access to them looks like this is now not necessary

/// Container holding the **Access Interface (AcI)** connection and metadata required for sorting and management.
///
/// This struct wraps the raw `AcI` with additional local state used by the `AcIManager` to track
/// reliability (failures), capacity, and registration order.
#[derive(Debug)]
pub struct AcIContainer {
    pub aci: AcI,

    /// The sequence number assigned at registration time, used for stable sorting.
    pub registration_index: usize,

    /// A counter of how many times operations on this AcI have failed.
    pub failures: u32,

    /// The total bandwidth or capacity available on the link to this AcI (does not mean free capacity).
    pub total_link_capacity: i64,

    /// The number of distinct link resources of the AcI.
    pub link_resource_count: usize,
}

impl AcIContainer {
    pub fn new(aci: AcI, registration_index: usize, total_link_capacity: i64, link_resource_count: usize) -> Self {
        Self { aci, registration_index, total_link_capacity, link_resource_count, failures: 0 }
    }
}

/// Manages a collection of **AcIs** within a specific **ADC**.
///
/// The `AcIManager` acts as a central registry and aggregator for distributed resources. It handles:
/// * Registration and deregistration of AcIs.
/// * Aggregation of system-wide metrics (Satisfaction, Load).
/// * Retrieval of AcIs based on specific ordering strategies (Random, Load-balanced, etc.).
///
/// # Distributed Context & Synchronization
///
/// This manager operates within a distributed Grid/VRM system. While `AcIManager` provides a local view
/// of the resources, operations performed on the contained `AcI` objects may involve network communication
/// with remote entities. Callers should assume that state changes (like load updates) require synchronization
/// with the remote AcIs.
#[derive(Debug)]
pub struct AcIManager {
    /// The ID of the ADC owning this manager.
    adc_id: AdcId,

    /// Map of `AciId` to their container wrappers.
    acis: HashMap<AciId, AcIContainer>,

    /// The aggregated sum of link capacities of all registered AcIs (does not mean free capacity).
    total_link_capacity: i64,

    /// The aggregated sum distinct link resources of all registered AcIs.
    link_resource_count: usize,

    /// Monotonic counter used to assign `registration_index` to new AcIContainer's.
    registration_counter: usize,
}

impl AcIManager {
    pub fn new(adc_id: AdcId, aci_set: HashSet<AcI>) -> Self {
        let mut acis = HashMap::with_capacity(aci_set.len());
        let mut registration_counter = 0;
        let mut aci_manager_total_link_capacity = 0;
        let mut aci_manager_link_resource_count = 0;

        for aci in aci_set {
            let aci_id = aci.id.clone();
            let total_link_capacity = aci.get_total_link_capacity();
            let link_resource_count = aci.get_link_resource_count();

            aci_manager_total_link_capacity += total_link_capacity;
            aci_manager_link_resource_count += link_resource_count;

            let container = AcIContainer::new(aci, registration_counter, total_link_capacity, link_resource_count);
            registration_counter += 1;

            acis.insert(aci_id, container);
        }

        AcIManager {
            adc_id,
            acis,
            total_link_capacity: aci_manager_total_link_capacity,
            link_resource_count: aci_manager_link_resource_count,
            registration_counter,
        }
    }

    /// Increments and returns the next available registration counter.
    pub fn get_new_registration_counter(&mut self) -> usize {
        let current = self.registration_counter;
        self.registration_counter += 1;
        return current;
    }

    /// Calculates the average link speed across all registered resources.
    pub fn get_average_link_speed(&self) -> f64 {
        if self.link_resource_count == 0 {
            return 0.0;
        }

        return self.total_link_capacity as f64 / self.link_resource_count as f64;
    }

    /// Registers a new **AcI** with the manager.
    ///
    /// # Arguments
    /// * `aci` - The `AcI` instance to add.
    ///
    /// # Returns
    /// * `true` - If the AcI was successfully added.
    /// * `false` - If the AcI ID already exists or if an insertion error occurred (integrity compromised).
    pub fn add_aci(&mut self, aci: AcI) -> bool {
        if self.acis.contains_key(&aci.id) {
            log::error!(
                "Process of adding a new AcI to the AciManger failed. It is not allowed to add the same aci multiple times. Please first delete the AcI: {}.",
                aci.id
            );
            return false;
        }

        let aci_id = aci.id.clone();
        let total_link_capacity = aci.get_total_link_capacity();
        let link_resource_count = aci.get_link_resource_count();
        let registration_index = self.get_new_registration_counter();

        let container = AcIContainer::new(aci, registration_index, total_link_capacity, link_resource_count);

        if self.acis.insert(aci_id.clone(), container).is_none() {
            return true;
        } else {
            log::error!(
                "Error happened in the process of adding a new AcI: {} to the AciManager (Adc: {}). The AciManger is now compromised.",
                aci_id,
                self.adc_id
            );
            return false;
        }
    }

    /// Removes an **AcI** from the manager by its ID.
    ///
    /// Updates the total link capacity and link resource counts upon successful removal.
    ///
    /// # Arguments
    /// * `aci_id` - The identifier of the AcI to remove.
    ///
    /// # Returns
    /// * `true` - If the AcI was found and removed.
    /// * `false` - If the AcI ID was not found.
    pub fn delete_aci(&mut self, aci_id: AciId) -> bool {
        let container = self.acis.remove(&aci_id);
        match container {
            Some(container) => {
                self.total_link_capacity -= container.total_link_capacity;
                self.link_resource_count -= container.link_resource_count;
                return true;
            }
            None => {
                log::error!(
                    "The process of deleting the AcI: {} form AciManager (Adc: {}). Failed, because the AciId was not present in the AciManager.",
                    aci_id,
                    self.adc_id
                );
                return false;
            }
        }
    }

    /// Returns a list of all registered AcI IDs in **random order**.
    ///
    /// # Returns
    /// A `Vec<AciId>` where the AciIds are in random order.
    pub fn get_random_ordered_acis(&self) -> Vec<AciId> {
        let mut keys: Vec<AciId> = self.acis.keys().cloned().into_iter().collect();
        keys.shuffle(&mut rng());
        return keys;
    }

    /// Returns a list of registered AcI IDs sorted according to the specified strategy.
    /// If strict ordering is not required, `get_random_ordered_acis` is preferred for performance.
    ///
    /// # Returns
    /// A `Vec<AciId>` sorted based on the comparator provided by `AcIOrder`.
    pub fn get_ordered_acis(&self, request_order: AcIOrder) -> Vec<AciId> {
        let comparator = request_order.get_comparator();
        let mut acis_vec: Vec<&AcIContainer> = self.acis.values().collect();

        acis_vec.sort_unstable_by(|a, b| comparator(a, b));

        let sorted_keys: Vec<AciId> = acis_vec.into_iter().map(|container| container.aci.id.clone()).collect();
        return sorted_keys;
    }

    /// Calculates the average **Satisfaction Score** (0.0 to 1.0) for the current schedule within a specific time window.
    /// This method queries all connected AcIs and calculates the capacity-weighted average satisfaction.
    ///
    /// # Arguments
    /// * `start` - The start of the time window.
    /// * `end` - The end of the time window.
    /// * `shadow_schedule_id` - Optional ID. If provided, calculates based on the specified shadow schedule; otherwise uses the master schedule.
    ///
    /// # Returns
    /// A `f64` value between 0.0 (worst case) and 1.0 (best case). Returns 0.0 if total capacity is 0.
    pub fn get_satisfaction(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        log::debug!(
            "ADC: {} requests satisfaction of all AcIs with the ShadowScheduleId: {:?} the time window start: {} to end: {}",
            self.adc_id,
            shadow_schedule_id.clone(),
            start,
            end
        );

        let mut satisfaction_sum = 0.0;
        let mut total_capacity = 0.0;

        for (id, container) in self.acis.iter_mut() {
            let satisfaction = container.aci.get_satisfaction(start, end, shadow_schedule_id.clone());

            if satisfaction < 0.0 {
                log::debug!(
                    "Satisfaction of AcI is not allowed to be negative. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                let cap = container.aci.get_total_node_capacity() as f64;
                satisfaction_sum += satisfaction * cap;
                total_capacity += cap;
            }
        }

        return if total_capacity > 0.0 { satisfaction_sum / total_capacity } else { 0.0 };
    }

    /// Calculates the system-wide **Satisfaction Score** (0.0 to 1.0) across the full range of every schedule.
    /// This method queries all connected AcIs and calculates the capacity-weighted average.
    ///
    /// # Behavioral Note
    /// **Network AcIs:** This calculation generally excludes network AIs if their satisfaction/fragmentation
    /// functions are not implemented (returning -1). These are filtered out to prevent skewing the system metric.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - Optional ID. If provided, calculates based on the specified shadow schedule.
    ///                          (If None utilize master schedule)
    ///
    /// # Returns
    /// A `f64` value between 0.0 (worst case) and 1.0 (best case).
    pub fn get_system_satisfaction(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> f64 {
        log::debug!("ADC: {} requests system satisfaction of all AcIs with the ShadowScheduleId: {:?}.", self.adc_id, shadow_schedule_id.clone());

        let mut satisfaction_sum = 0.0;
        let mut total_capacity = 0.0;

        for (id, container) in self.acis.iter_mut() {
            let satisfaction = container.aci.get_system_satisfaction(shadow_schedule_id.clone());
            if satisfaction < 0.0 {
                log::debug!(
                    "System satisfaction of AcI is not allowed to be negative. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                let cap = container.aci.get_total_node_capacity() as f64;
                satisfaction_sum += satisfaction * cap;
                total_capacity += cap;
            }
        }

        return if total_capacity > 0.0 { satisfaction_sum / total_capacity } else { 0.0 };
    }

    /// Computes the **Load Metric** for a specific time range.
    /// This method aggregates the load of all AcIs.
    /// **Note:** Only jobs submitted via this ADC are typically counted; actual load on the physical resource
    /// may be higher due to local jobs or other ADCs.
    ///
    /// # Arguments
    /// * `start` - Start of the analysis window in seconds (VRM Time).
    /// * `end` - End of the analysis window in seconds (VRM Time).
    /// * `shadow_schedule_id` - Optional ID for shadow schedule analysis (If None utilize master schedule).
    ///
    /// # Returns
    /// A `LoadMetric` struct containing utilization, start/end times, and capacity details.
    pub fn get_load_metric(&mut self, start: i64, end: i64, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        let mut total_possible_reserved_capacity = 0.0;
        let mut total_average_reserved_capacity = 0.0;
        let mut earliest_start = i64::MAX;
        let mut latest_end = i64::MIN;
        let mut num_of_valid_acis = 0;

        for (id, container) in self.acis.iter_mut() {
            let load_matic = container.aci.get_load_metric_up_to_date(start, end, shadow_schedule_id.clone());

            if load_matic.start_time < 0 {
                log::debug!(
                    "Get Load Metric of AcI with negative start time is not allowed. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                total_average_reserved_capacity += load_matic.avg_reserved_capacity;
                total_possible_reserved_capacity += load_matic.possible_capacity;
                num_of_valid_acis += 1;

                if earliest_start > load_matic.start_time {
                    earliest_start = load_matic.start_time;
                }

                if latest_end < load_matic.end_time {
                    latest_end = load_matic.end_time;
                }
            }
        }

        let mut utilization: f64 = 0.0;
        if total_possible_reserved_capacity > 0.0 {
            utilization = total_average_reserved_capacity / total_possible_reserved_capacity;
        }

        if num_of_valid_acis > 0 {
            return LoadMetric::new(
                earliest_start,
                latest_end,
                total_average_reserved_capacity / num_of_valid_acis as f64,
                total_possible_reserved_capacity / num_of_valid_acis as f64,
                utilization,
            );
        } else {
            return LoadMetric::new(earliest_start, latest_end, 0.0, 0.0, utilization);
        }
    }

    /// Computes the **Load Metric** for the entire simulation timeline.
    /// Aggregates metrics from all valid AcIs to provide a high-level view of system utilization.
    ///
    /// # Arguments
    /// * `shadow_schedule_id` - Optional ID for shadow schedule analysis (If None utilize master schedule).
    ///
    /// # Returns
    /// A `LoadMetric` representing the average reserved capacity and utilization across the simulation.
    pub fn get_simulation_load_metric(&mut self, shadow_schedule_id: Option<ShadowScheduleId>) -> LoadMetric {
        let mut total_possible_reserved_capacity = 0.0;
        let mut total_average_reserved_capacity = 0.0;
        let mut earliest_start = i64::MAX;
        let mut latest_end = i64::MIN;
        let mut num_of_valid_acis = 0;

        for (id, container) in self.acis.iter_mut() {
            let load_matic = container.aci.get_simulation_load_metric(shadow_schedule_id.clone());

            if load_matic.start_time < 0 {
                log::debug!(
                    "Get Load Metric of AcI with negative start time is not allowed. ADC: {}, AcIs:  {} with ShadowScheduleId: {:?}",
                    self.adc_id,
                    id,
                    shadow_schedule_id
                );
            } else {
                total_average_reserved_capacity += load_matic.avg_reserved_capacity;
                total_possible_reserved_capacity += load_matic.possible_capacity;
                num_of_valid_acis += 1;

                if earliest_start > load_matic.start_time {
                    earliest_start = load_matic.start_time;
                }

                if latest_end < load_matic.end_time {
                    latest_end = load_matic.end_time;
                }
            }
        }

        let mut utilization: f64 = 0.0;
        if total_possible_reserved_capacity > 0.0 {
            utilization = total_average_reserved_capacity / total_possible_reserved_capacity;
        }

        if num_of_valid_acis > 0 {
            return LoadMetric::new(
                earliest_start,
                latest_end,
                total_average_reserved_capacity / num_of_valid_acis as f64,
                total_possible_reserved_capacity / num_of_valid_acis as f64,
                utilization,
            );
        } else {
            return LoadMetric::new(earliest_start, latest_end, 0.0, 0.0, utilization);
        }
    }
}
