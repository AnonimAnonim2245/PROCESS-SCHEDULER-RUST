# PROCESS-SCHEDULER-RUST
This was done as homework for the course "Computer programming and programming languages 4", course led by Alexandru Radovici. Within this homework I had to make a process scheduler in Rust utilizing three methods: round-robin, round-robin with priority queue and cfs.

Process Management: allows adding, removing and managing processes in a queue.

Time Synchronization: Synchronizes the time of each process according to the amount of time allocated.

Change Process Status: Allows to change the status of processes (e.g. from Ready to Running).

Sleep Management: Implements logic for sleeping processes.

The signal method is used to manage signals between processes. When a process issues a signal (such as completing a task or needing to wake up other processes), the signal is used to update the state of the processes waiting for that event. The main functions are:

Sorting Processes: Sorts processes by ID and waiting status. Process Status Update: Changes the status of processes waiting for an event (signaled by the signal) from Waiting to Ready. Process Queue Reorder: Updates the order of processes in the queue to reflect status changes.

The wait function is used to put a process in the Waiting state. It is useful when a process must wait for an event or condition before continuing. It works like this:

Set Waiting State: Changes the state of the current process to Waiting. Time Management: Updates the process time for the waiting period.

Change Process State: Implements the logic required to change process states between Ready, Running, Waiting and other states.

Synchronize Timings to Round Robin: A key function is to synchronize timings for processes, ensuring that each process is updated accordingly.

Manage Fork and Exit Syscalls: Allows you to create and terminate processes, updating the status and order of processes in the queue.

Manage vruntime for each process. This is a crucial feature of CFS, where each process is scheduled by its virtual runtime.

Use find_smallest_vm_run and select_smallest_vm_run functions to identify the processes with the smallest vruntime.

Synchronize timings to CFS: The syncronize_timings method updates the timings for all processes, ensuring that vruntime and other timings are maintained correctly.

Time Amount Management: Adjusts the amount of time according to the number of processes and their states.

Regarding unwrap, there are conditions that check if the list is not empty and if our item is greater than or equal to zero for NonZeroUsize. Don't panic because the values are typical for a NonZeroUsize, greater than zero.

Various comments are added as to why it doesn't panic on unwrap
