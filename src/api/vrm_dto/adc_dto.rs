use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ADCDto {
    id: String,
    scheduler_typ: String,
    request_order: String,
    num_of_slots: i64,
    slot_width: i64,
    timeout: i64,
    max_optimization_time: i64,
    reject_new_reservations_at: i64,
}
