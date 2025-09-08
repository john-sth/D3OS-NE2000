// =============================================================================
// FILE        : mod.rs
// MODULE      : ne2k
// AUTHOR      : Johann Spenrath
// DESCRIPTION : define files which belong to the module
// =============================================================================
//
// NOTES:
// =============================================================================

// add benchmark functions for testing in kernel space
pub mod benchmark;
// constants for addressing the registers of the NE2000
pub mod consts;
// Trait Implementations for smoltcp
pub mod device_smoltcp;
// main driver functionalities
pub mod ne2000;
