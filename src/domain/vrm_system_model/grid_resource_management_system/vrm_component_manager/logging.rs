impl VrmComponentManager {
    pub fn log_stat(&mut self, command: String, reservation_id: ReservationId, arrival_time_at_aci: i64) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let processing_time = self.simulator.get_system_time_s() - arrival_time_at_aci;

        if let Some(res_handle) = self.reservation_store.get(reservation_id) {
            let (start, end, res_name, capacity, workload, state, proceeding, num_tasks) = {
                let res = res_handle.read().unwrap();

                let start = res.get_base_reservation().get_assigned_start();
                let end = res.get_base_reservation().get_assigned_end();
                let name = res.get_base_reservation().get_name().clone();
                let cap = res.get_base_reservation().get_reserved_capacity();
                let workload = res.get_base_reservation().get_task_duration() * cap;
                let state = res.get_base_reservation().get_state();
                let proceeding = res.get_base_reservation().get_reservation_proceeding();

                let mut tasks = 1;
                if res.is_workflow() {
                    tasks = res.as_workflow().unwrap().get_all_reservation_ids().len()
                }

                (start, end, name, cap, workload, state, proceeding, tasks)
            };

            let rms_load_metric = self.get_load_metric(start, end, None);

            let node_utilization = rms_load_metric.node_load_metric.as_ref().map(|n| Some(n.utilization)).unwrap_or(None);

            let node_possible_capacity = rms_load_metric.node_load_metric.as_ref().map(|n| Some(n.possible_capacity)).unwrap_or(None);

            let network_utilization = rms_load_metric.link_load_metric.as_ref().map(|n| Some(n.utilization)).unwrap_or(None);

            let network_possible_capacity = rms_load_metric.link_load_metric.as_ref().map(|n| Some(n.possible_capacity)).unwrap_or(None);

            tracing::info!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished",
                ComponentType = %self.adc_id.clone(),
                NodeComponentUtilization = node_utilization,
                NodeComponentCapacity = node_possible_capacity,
                NetworkComponentUtilization = network_utilization,
                NetworkComponentCapacity = network_possible_capacity,
                ComponentFragmentation = self.get_system_satisfaction(None),
                ReservationName = %res_name,
                ReservationCapacity = capacity,
                ReservationWorkload = workload,
                ReservationState = ?state,
                ReservationProceeding = ?proceeding,
                NumberOfTasks = num_tasks,
                Command = command,
                ProcessingTime = processing_time,
            );
        } else {
            // Handling in case reservation is missing (e.g. deleted/cleaned up)

            tracing::warn!(
                target: ANALYTICS_TARGET,
                Time = now,
                LogDescription = "AcI Operation finished (Reservation Missing/Deleted)",
                ComponentType = %self.adc_id,
                ReservationId = ?reservation_id,
                Command = command,
                ProcessingTime = processing_time,
            );
        }
    }
}