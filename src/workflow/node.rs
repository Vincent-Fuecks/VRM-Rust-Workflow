use crate::reservation::NodeReservation;

pub struct WorkflowNode {
    pub reservation: NodeReservation,
    pub incoming_dependencies: Vec<String>,
    pub outgoing_dependencies: Vec<String>,
}