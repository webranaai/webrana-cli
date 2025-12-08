//! Crew - Custom AI Personas
//!
//! Create and manage custom AI personas with specialized behaviors,
//! system prompts, and tool permissions.

mod persona;
mod manager;

pub use persona::{Crew, CrewConfig, CrewPermissions, CrewTemplate};
pub use manager::CrewManager;
