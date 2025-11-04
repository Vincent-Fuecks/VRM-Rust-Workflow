use crate::workflow::link_reservation::LinkReservation;

pub enum DependencyType {
    Data { size: u64 },
    Sync { bandwidth: u64 },
}

pub struct Dependency {
    pub link_reservation: LinkReservation,
    pub kind: DependencyType,
    pub source_id: String,
    pub target_id: String,
    pub port_name: String,
}

impl Dependency {

    // pub fn recompute_booking_interval(&mut self, source_node: &WorkflowNode, target
    //     match self.kind {
    //         DependencyType::Data { .. } => {
        
    //         self.link_reservation.booking_interval_start = 
    //         self.link_reservation.booking_interval_end =
    //         }
    //         DependencyType::Sync { .. } => {
    //             self.link_reservation.booking_interval_start = 
    //             self.link_reservation.booking_interval_end = 
    //         }
    // }
}
