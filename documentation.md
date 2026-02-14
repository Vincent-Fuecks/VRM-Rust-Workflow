# About the System
The system I am building is called VRM. This system revisers workflows from different clients. Than distributes this system these workflow subtasks to connected Clusters.

These Clusters are hierarchically organised by the VRM system. The system is build like a tree and consist of ADCs and AcIs. An ADC is a manager, that manages multiple cluster directly or indirectly. But this system does not know the detail over the underlying system. Contains Workflow Scheduler etc.
The AcI is the Scheduler for the connected Cluster, which task is currently running, what task could I submit next etc.

The system is build the following:
The root is the master ADC. The inner nodes of the tree are only other ADCs and the leafs are only AcIs. Means, an ADC is managing other ADCS and AcIs, but an AcI is only connected to his slurm cluster.


The ADC and AcI run in there own Threads.
All ADCs are running on one machine, but all AcI run on different machines (where the slurm cluster is located)(But this part is not implemented yet).

# Core Hierachy
- System is organised in a Tree Structure to allow scalabel management of heterogeneous resources

## Master ADC (Root)
- Entry point of all client workflows
- Coordinates the top-level distribution of subtasks

## ADC (Inner Nodes)
- Acts as a regional manager
- Abstraction Layer: Manages underlying resources (other ADCs or AcIs) without needing to know the low-level details of the physical cluster
- Contains internal Workflow Schedulers to optimize task distribution among its children.

## AcI (Leaf)
- The actual interface to a physical computing resource (e.g., a Slurm Cluster)
- Manages Communication with cluster, 
- Replicates the schedule of the Rms of the cluster --> Performes shadow scheduling ot provided a advanded scheudling system, even if the underlying system is working with a queuing system for example

# Execution Model 
Currently is it a multi-threaded model on a contralized machine 
- Dedicated Threads: Exery ADC and AcI runis in its own thread. 

# Communication Pattern
Inter-Node: ADCs and AcIs communicate across thread boundaries using message passing or shared memory structures (ReservationStrore).

AcI to SlurmRms: AcIs use a Blocking I/O pattern. Since each AcI has its own thread, it can safely wait for Slurm API responses without impacting system-wide performance.

# Remarks: 
- In the java program in  ../src/vrm/util/LoadBuffer.java exists a prototype implementation of a hiracical loadbuffer
- In the java version was the Idea present of marking resources as up or down. However, the system was never utilized
- Idea: Utilizing a Repository Pattern for clear Resource access 
- Security problem in reservation.rs is it possible to access all stored reservations of the system, if the SlottedSchedule, AcI or Adc submits the corresponding ReservationId (also if they are not managing this reservation) --> necessary to trust all these components --> Solution: Is submitted ReservationId in reservations HashSet if provide access else reject access
- The ReservationStore enables better management of the Reservation via the ReservationId's -- However, now that only one ReservationStore for each Schedule exists is the process requesting a prob request changed because the Reservation object contains all ReservationId candidates, however the state of these reservations is not set to ProbAnswer or? 
- Currently the resources of an RMS are also own by it. ADC and ACI must have read-only access to these resources. The information flow is currently like this: ADC --> ACI --> RMS --> Resources. If more kinds of these requests are required a repository pattern like for reservations should be implemented (like reservation_store.rs)