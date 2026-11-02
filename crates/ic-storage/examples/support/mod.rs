//! Passive fixture protocol shared with the installed runner.
/// Bounded Wire identities and passive fixture DTOs.
pub mod qualification_protocol;

// Binary-owned runtime support and passive qualification protocol.
/// Runtime instance owned independently by each fixture binary.
pub mod host;
/// Bounded Wire identities and passive IO fixture DTOs.
pub mod io_protocol;
