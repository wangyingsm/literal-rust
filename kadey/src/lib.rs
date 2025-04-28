pub mod core;

#[cfg(test)]
const ID_BYTES_LENGTH: usize = 4;
#[cfg(not(test))]
const ID_BYTES_LENGTH: usize = 20;

const K_REPLICATIONS: usize = 20;
const ALPHA_PARALLEL: usize = 3;
