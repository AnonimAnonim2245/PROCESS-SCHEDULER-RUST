//! Implement the schedulers in this module
//!
//! You might want to create separate files
//! for each scheduler and export it here
//! like
//!
//! ```ignore
//! mod scheduler_name
//! pub use scheduler_name::SchedulerName;
//! ```
//!

// TODO delete this example
mod empty;
pub use empty::Empty;
mod round_robin;
mod cfs;
pub use round_robin::RoundRobin;
pub use cfs::Cfs;
// TODO import your schedulers here
