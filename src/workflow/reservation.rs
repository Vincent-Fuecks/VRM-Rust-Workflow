/// TODO Some functinalities are still messing from the original implementation (like xml parser etc)
/// TODO Parser can save and load Reservations in xml format 
/// TODO How do I do I create unique ids for the reservations? 
/// TODO find better structure as UNIQUE_IDS
use serde::{Deserialize};
use std::collections::HashSet;
use std::sync::Mutex;
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::loader::parser::parse_joson_file;


// Utilized to store all currently utilized unique IDs for all reservations in the system
lazy_static! {
    static ref UNIQUE_RESERVATION_IDS: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
}

/// Defines the lifecycle state of a job reservation within the system.
///
/// This state tracks the progress of the reservation from initial request
/// through processing, commitment, and eventual completion or failure.
/// 
/// The order, from lowest commitment (0) to highest (6), is:
/// 1.  `Rejected`
/// 2.  `Deleted`
/// 3.  `Open`
/// 4.  `ProbeAnswer`
/// 5.  `ReserveAnswer`
/// 6.  `Committed`
/// 7.  `Finished`
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReservationState {
    /// The last request of the reservation was explicitly denied or failed.
    Rejected,
    /// The reservation has been successfully cancelled and removed from the system.
    Deleted,
    /// The reservation is newly created and has not yet been submitted to any processor.
    Open,
    /// The state represents a successful response to a probing (availability) request.
    ProbeAnswer,
    /// The state represents a successful response to a resource reservation request.
    ReserveAnswer,
    /// The reservation has been confirmed and resources are formally allocated.
    Committed,
    /// The execution phase of the job linked to this reservation has been finished successfully.
    Finished,
}

impl ReservationState {
    /// Returns `true` if this reservation state has reached a commitment level equal to or higher
    /// than the `other` state.
    ///
    /// The commitment order is defined by the variant declaration order (0-6).
    /// For example, `Committed` is at least (`>=`) `ProbeAnswer`, but not `Finished`.
    pub fn is_at_least(&self, other: Self) -> bool {
        self.cmp(&other) != std::cmp::Ordering::Less
        // Alternatively, leveraging PartialOrd directly:
        // self >= &other
    }
}

/// Specifies the process state the reservation is currently in. 
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum ReservationProceeding {
    /// Only perform the initial **probe** request to check availability.
    Probe,
    /// Send only a reserve request and quit then. Do not cancel the reservation.
    Reserve,
    /// Commit the reservation
    Commit,
    /// Reserve the reservation, but delete it within the commit timeout
    Delete,
}

/// A reservation represents any kind of job to be executed in the Grid. A 
/// reservation is used during all steps, so even if it is technically 
/// just a "reservation request" or if the reservation was rejected
/// 
/// The core data structure representing a resource reservation request.
///
/// This object is used throughout the entire lifecycle of a job, from the initial
/// request to final execution, and represents the current status of the
/// resource allocation.
///
/// Note that in a distributed setup, multiple objects may represent the same
/// reservation. They are identified by their unique `id` and rely on a
/// consistent synchronization across components.
#[derive(Debug, Clone)]
pub struct ReservationBase {
    /// --- IDENTITY & STATE ---

    /// A unique identifier assigned to the reservation upon creation.
    pub id: String,

    /// The current state of this specific reservation instance.
    ///
    /// This may represent only the local state and might not perfectly
    /// reflect the global state of the reservation.
    pub state: ReservationState,

    /// The client's instruction on how far the reservation process should proceed.
    pub proceeding: ReservationProceeding,

    /// TODO Is this redandent???
    pub used_aci_hierarchy: Vec<String>,
    

    /// --- TIME WINDOWS (All fields are in seconds) ---

    /// The time  this job arrived in the system.
    pub arrival_time: i64,

    /// The earliest possible start time for the job.
    pub booking_interval_start: i64,

    /// The latest possible end time for the job.
    pub booking_interval_end: i64,

    /// The scheduled start time of the job. Must be within the booking interval.
    pub assigned_start: i64,
    
    /// The scheduled end time of the job. Must be within the booking interval.
    pub assigned_end: i64,


    // --- RESOURCE & MOLDING ---

    /// Used for fragmentation calculation; a tolerance delta value.
    pub frag_delta: f32,
    
    /// The requested and reserved duration of the job (in seconds).
    pub job_duration: i32,

    /// The requested and reserved capacity of this job. 
    /// The capacity is measured in a unit according to the job type 
    /// e.g. number of CPUs for NodeReservation or kBit/s Bandwidth for LinkReservation 
    pub reserved_capacity: i32,
    
    /// If true, the `job_duration` and `reserved_capacity` are adjustable (moldable)
    /// during the reservation process to fit available resources.
    pub moldable: bool,

    /// Internal field: The total required work, calculated as (`reserved_capacity` * `job_duration`).
    ///
    /// Used internally to adjust capacity and duration while preserving the total work required
    /// for moldable reservations.
    moldable_capacity: i32, 
}

/// See ReservationBase for detailed information regarding the fields. 
/// The following struct is a Data Transfer Object (DTO) for deserialization. 
/// This struct mathces the shape of teh incomin Json modeling reservations. 
#[serde(rename_all = "camelCase")]
#[derive(Debug, Deserialize)]
pub struct ReservationBaseDto {
    /// --- IDENTITY & STATE ---
    pub id: Option<String>,
    pub proceeding: ReservationProceeding,
    pub used_aci_hierarchy: Vec<String>,
    

    /// --- TIME WINDOWS (All fields are in seconds) ---
    pub arrival_time: i64,
    pub booking_interval_start: i64,
    pub booking_interval_end: i64,
    pub assigned_start: i64,
    pub assigned_end: i64,


    // --- RESOURCE & MOLDING ---
    pub frag_delta: f32,
    pub job_duration: i32,
    pub reserved_capacity: i32,
    pub moldable: bool,
    moldable_capacity: i32, 
}

impl ReservationBase {
    pub fn new(path_to_reservation: &str) -> Self {
        let dto = parse_joson_file::<ReservationBaseDto>(path_to_reservation)
            .expect("Failed to load or parse reservation file.");
        
        Self::form_dto(dto)

    }

    pub fn form_dto(dto: ReservationBaseDto) -> Self {
        let id = Self::get_final_id(dto.id); 
        let moldable_capacity = dto.reserved_capacity * dto.job_duration; 

        ReservationBase {
            id: id,
            state: ReservationState::Open, 
            proceeding: dto.proceeding,
            used_aci_hierarchy: dto.used_aci_hierarchy,
            arrival_time: dto.arrival_time,
            booking_interval_start: dto.booking_interval_start,
            booking_interval_end: dto.booking_interval_end,
            assigned_start: dto.assigned_start,
            assigned_end: dto.assigned_end,
            frag_delta: dto.frag_delta,
            job_duration: dto.job_duration,
            reserved_capacity: dto.reserved_capacity,
            moldable: dto.moldable,
            moldable_capacity: moldable_capacity,
        }
    }

    fn get_final_id(optional_id: Option<String>) -> String {
        let mut id_set = match UNIQUE_RESERVATION_IDS.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                eprintln!("Mutex was poisoned. Recovering data.");
                poisoned.into_inner()
            }
        };

        // Check if the provided ID is valid and unused
        if let Some(ref id) = optional_id {
            if !id_set.contains(id) {
                // ID was provided and is not in use.
                id_set.insert(id.clone());
                return id.clone();
            } else {
                // ID was provided but is already in use.
                // TODO Logger Log Warning, that other Id was used to create reservation
                eprintln!("Warning: Provided ID '{}' is already in use. Generating a new one.", id);
            }
        }

        let mut new_id = Uuid::new_v4().to_string();
        
        // If new generated Id is allready in use create a new one.
        while id_set.contains(&new_id) {
            new_id = Uuid::new_v4().to_string();
        }
        
        id_set.insert(new_id.clone());
        new_id
    }

    /// Recalculates and updates the internal `moldable_capacity` based on the current
    /// `reserved_capacity` and `job_duration`.
    fn update_moldable_capacity(&mut self) {
        self.moldable_capacity = self.reserved_capacity * self.job_duration;
    }


    /// Sets a new job duration and recalculates the internal `moldable_capacity` based on this change.
    ///
    /// This method performs an **inherent change** in the job size (total work). To adjust the
    /// duration while keeping the total work constant for moldable jobs, use [`adjust_job_duration`].
    pub fn set_job_duration(&mut self, job_duration: i32) {
        self.job_duration = job_duration;
        self.moldable_capacity = self.moldable_capacity * self.job_duration;
    }


    /// Adjusts the job duration and recalculates the requested capacity for **moldable reservations**.
    ///
    /// This operation changes the duration and capacity such that the total required work
    /// (`moldable_capacity`) remains constant. For inherent changes to the job size, use [`set_job_duration`].
    pub fn adjust_job_duration(&mut self, duration: i32) {
        if !self.moldable {
            // TODO Logging --> Logger.error("adjustJobDuration for non moldable job " + this);
            eprintln!("Warning: adjust_job_duration called for non-moldable job. Proceeding with adjustment.");
        }

        self.job_duration = duration.max(1);
        self.reserved_capacity = (self.moldable_capacity / self.job_duration).max(1);
    }

    /// Adjusts the reserved capacity and recalculates the job duration for **moldable reservations**.
    ///
    /// This operation changes the capacity and duration such that the total required work
    /// (`moldable_capacity`) remains constant. For inherent changes to the job size, use
    /// [`set_reserved_capacity`].
    ///
    /// The capacity unit is specific to the job type (e.g., CPUs or Bandwidth).
    pub fn adjust_capacity(&mut self, capacity: i32) {
        if capacity != self.reserved_capacity {
            if !self.moldable {
                // TODO Logging --> Logger.error("adjustCapacity for non moldable job " + this);
                eprintln!("Warning: adjust_capacity called for non-moldable job. Proceeding with adjustment.");
            }

            self.reserved_capacity = capacity.max(1);
            self.job_duration = (self.moldable_capacity / self.reserved_capacity).max(1);
        }
    }

    /// Sets a new reserved capacity and recalculates the internal `moldable_capacity` based on this change.
    ///
    /// This method performs an **inherent change** in the job size (total work). To adjust the
    /// capacity while keeping the total work constant for moldable jobs, use  [`adjust_capacity`].
    ///
    /// The capacity unit is specific to the job type (e.g., CPUs or Bandwidth).
    pub fn set_reserved_capacity(&mut self, reserved_capacity: i32) {
        self.reserved_capacity = reserved_capacity;
        self.moldable_capacity = self.reserved_capacity * self.job_duration;
    }

    /// Determines if two reservations are considered logically equal based on their unique ID.
    ///
    /// Two reservations are equal if they carry the same unique ID (`id`). If both reservations
    /// TODO Maybe later None possible, but should be prevented
    /// TODO All revervations should have a valid unique ID 
    pub fn equal_name(&self, other: &Self) -> bool {
        if std::ptr::eq(self, other) {
            return true;
        }

        // Since `id` is now a non-optional String, we can just compare them directly.
        self.id == other.id
    }
}