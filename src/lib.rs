//! Concrete mechanisms shared by named consumer games: presentation, screen
//! capture and the agent seat, behind `render` and `seat`.
//!
//! Seeded randomness, hover, recoverable file replacement, asset reads and the
//! held-seat client live in packages of their own that cannot depend on Bevy:
//! `unfolded-random`, `unfolded-hover`, `unfolded-record-replacement`,
//! `unfolded-assets` and `unfolded-seat-client`.
#[cfg(feature = "render")]
pub mod camera;
#[cfg(feature = "render")]
pub mod capture;
#[cfg(feature = "render")]
pub mod card;
#[cfg(feature = "render")]
pub mod label_anchor;
#[cfg(feature = "render")]
pub mod loading;
#[cfg(feature = "render")]
pub mod printed;
#[cfg(feature = "seat")]
pub mod seat;
#[cfg(feature = "render")]
pub mod token;
#[cfg(feature = "render")]
pub mod window_size;
