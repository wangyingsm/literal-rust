pub mod core;
pub mod rpc;

#[cfg(test)]
pub const ID_BYTES_LENGTH: usize = 4;
#[cfg(not(test))]
pub const ID_BYTES_LENGTH: usize = 20;

pub const K_REPLICATIONS: usize = 20;
pub const ALPHA_PARALLEL: usize = 3;
