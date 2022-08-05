#[rustfmt::skip]
#[path = "sf.substreams.v1.rs"]
mod pbsubstreams;

#[rustfmt::skip]
#[path = "aptos.stats.v1.rs"]
mod pbstats;

// Kind of bad because we mix stuff from different modules merging everything together
// but I'm a bit unsure about how to fix this properly for now.
pub use pbstats::*;
pub use pbsubstreams::*;
