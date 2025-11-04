use crate::workflow::reservation::ReservationBase;

/**
 * A reservation of network capacity between two router in a managed 
 * network.
 * 
 * There are two typical use cases for link reservations: Transfer of
 * a file between two sites or reserving bandwidth between co-allocated
 * reservations for the short term communication and coordination between 
 * the jobs. In the first case the reservation is moldable (see {@link #isMoldable()})
 * and duration may be changed according to the available bandwidth. In the second case
 * the given bandwidth has to be provided during the specified duration.
 * 
 * The start and end router are specified by their unique name within the network.
 * 
 * In {@link Workflow}s the link reservations are mostly implicit specified and are
 * created during the scheduling process.
 * 
 * @see LinkResource
 * @see NullBroker
 */
pub struct LinkReservation {
    pub reservation_base: ReservationBase, 
    pub start_router_id: i32, 
    pub end_router_id: i32,
    pub start_aci_id: i32, 
    pub start_aci_end: i32,  
}

// impl Default for LinkReservation {

// }