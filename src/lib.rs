//! Concrete mechanisms shared by named consumer games: presentation, screen
//! capture and the agent seat, behind `render` and `seat`.
//!
//! Seeded randomness, hover, recoverable file replacement, asset reads, the held-seat
//! client and the agent players' client live in packages of their own that cannot
//! depend on Bevy: `unfolded-random`, `unfolded-hover`, `unfolded-record-replacement`,
//! `unfolded-assets`, `unfolded-seat-client` and `unfolded-agent-players`.
#[cfg(feature = "seat")]
pub mod agent_transport;
#[cfg(feature = "render")]
pub mod camera;
#[cfg(feature = "render")]
pub mod capture;
#[cfg(feature = "render")]
pub mod card;
#[cfg(feature = "render")]
pub mod card_motion;
#[cfg(feature = "render")]
pub mod label_anchor;
#[cfg(feature = "render")]
pub mod loading;
#[cfg(feature = "render")]
pub mod pipeline;
#[cfg(feature = "render")]
pub mod printed;
#[cfg(feature = "seat")]
pub mod seat;
#[cfg(feature = "render")]
pub mod slab;
#[cfg(feature = "render")]
pub mod token;
#[cfg(feature = "render")]
pub mod window_size;
