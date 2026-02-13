# Note this is the VRM simulation system
- ADCs and AcIs all run on the same system

# Remarks: 
- In the java program in  ../src/vrm/util/LoadBuffer.java exists a prototype implementation of a hiracical loadbuffer
- In the java version was the Idea present of marking resources as up or down. However, the system was never utilized
- Idea: Utilizing a Repository Pattern for clear Resource access 
- Security problem in reservation.rs is it possible to access all stored reservations of the system, if the SlottedSchedule, AcI or Adc submits the corresponding ReservationId (also if they are not managing this reservation) --> necessary to trust all these components --> Solution: Is submitted ReservationId in reservations HashSet if provide access else reject access
- The ReservationStore enables better management of the Reservation via the ReservationId's -- However, now that only one ReservationStore for each Schedule exists is the process requesting a prob request changed because the Reservation object contains all ReservationId candidates, however the state of these reservations is not set to ProbAnswer or? 
- Currently the resources of an RMS are also own by it. ADC and ACI must have read-only access to these resources. The information flow is currently like this: ADC --> ACI --> RMS --> Resources. If more kinds of these requests are required a repository pattern like for reservations should be implemented (like reservation_store.rs)