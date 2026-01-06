use std::collections::{HashMap, HashSet};

use crate::domain::vrm_system_model::grid_resource_management_system::aci::AcI;
use crate::domain::vrm_system_model::utils::id::{AciId, AdcId};

/// Container holding the AcI connection and metadata required for sorting/management.
#[derive(Debug)]
pub struct AcIContainer {
    pub aci: AcI,

    // Metadata for sorting strategies
    pub registration_index: usize,
    pub failures: u32,

    pub average_link_speed: f64,
    // pub last_known_metrics: LoadMetric,
}

impl AcIContainer {
    pub fn new(aci: AcI, registration_index: usize, average_link_speed: f64) -> Self {
        Self { aci, registration_index, average_link_speed, failures: 0 }
    }
}

#[derive(Debug)]
pub struct AcIManager {
    adc_id: AdcId,
    acis: HashMap<AciId, AcIContainer>,
    registration_counter: usize,
}

impl AcIManager {
    pub fn new(adc_id: AdcId, aci_set: HashSet<AcI>) -> Self {
        let acis: HashMap::new();
        let registration_counter = 0;

        for aci in aci_set.iter() {
            let aci_id = aci.id.clone();

            let container = AcIContainer::new(*aci, registration_counter, aci.get_average_network_speed());
            registration_counter += 1;

            acis.insert(aci_id, container);
        }

        AcIManager { adc_id, acis, registration_counter }
    }

    pub fn get_new_registration_counter(&mut self) -> usize {
        self.registration_counter += 1;
        return self.registration_counter - 1;
    }

    pub fn add_aci(&mut self, aci: AcI) -> bool {
        let aci_id = aci.id.clone();
        let container = AcIContainer::new(aci, self.get_new_registration_counter(), aci.get_average_network_speed());

        if self.acis.contains_key(&aci_id) {
            log::error!(
                "Process of adding a new AcI to the AciManger failed. It is not allowed to add the same aci multiple times. Please first delete the AcI: {}.",
                aci_id
            );
            return false;
        }

        if self.acis.insert(aci_id, container).is_none() {
            return true;
        }

        log::error!(
            "Error happened in the process of adding a new AcI: {} to the AciManager (Adc: {}). The AciManger is now compromised.",
            aci_id,
            self.adc_id
        );
        return false;
    }

    pub fn delete_aci(&mut self, aci_id: AciId) -> bool {
        if self.acis.remove(&aci_id).is_none() {
            return false;
            log::error!(
                "The process of deleting the AcI: {} form AciManager (Adc: {}). Failed, because the AciId was not present in the AciManager.",
                aci_id,
                self.adc_id
            )
        }
        return true;
    }

    /** Returns the list of registered AIs in no specific order. Use this
     *  method if you want to call all AIs.
     *
     * Naturally the AIs are sorted by the registration time, but no caller should
     *  rely on this assumption. Use {@link #getOrderedAis(long, long, AIOrder)} and
     *  {@link AIOrder#ORDER_START_FIRST} instead.
     *
     * @return A list of objects in no specific order containing a local copy/view of the AI schedule, some
     *         additional local informations and the connection object to call the AI.
     */
    pub fn get_unordered_acis(&self) -> Vec<AciId> {
        self.acis.keys().into_iter().collect()
    }

    /** Returns the list of registered AIs in the given order. Use this
     *  method if you need a list of a AIs to call one by one until a
     *  matching result is found.
     *
     *  If no order is needed, as all AIs should be called, use the much faster
     *  {@link #getUnorderedAis()} method.
     *
     * @param start some order take the state of the AI in the specific time
     *              of the request into account. This is the start time of the
     *              analyzed time frame. If no specific time frame should be analyzed,
     *              use {@link Simulator#TIME_NOT_SET}.
     * @param end   some order take the state of the AI in the specific time
     *              of the request into account. This is the end time of the
     *              analyzed time frame. If no specific time frame should be analyzed,
     *              use {@link Simulator#TIME_NOT_SET}.
     * @param requestOrder Identifier to mark the requested order. Use {@link ADCcore#requestOrder}
     *                     if in doubt.
     * @return A list of objects in no specific order containing a local copy/view of the AI schedule, some
     *         additional local informations and the connection object to call the AI.
     *
     * @see AIOrder#ORDER_START_FIRST
     * @see AIOrder#ORDER_NEXT
     * @see AIOrder#ORDER_LOAD
     * @see AIOrder#ORDER_REVERSE_LOAD
     * @see AIOrder#ORDER_SIZE
     * @see AIOrder#ORDER_REVERSE_SIZE
     */
    pub fn get_ordered_acis(start: i64, end: i64, request_order: AcIOrder) {}
}
