#![forbid(unsafe_code)]
#![warn(missing_docs)]

/*!
 * Implementation of the Per-Query Preprocessing variant of RAID-PIR as
 * described by Günther et al.
 *
 * Should be considered academic and not used for production.
 */

pub mod client;
pub mod server;
pub mod types;
pub mod util;
