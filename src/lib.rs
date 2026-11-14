//! Concrete mechanisms shared by named consumer games: presentation, screen
//! capture, the agent seat, seeded randomness, and recoverable file replacement.
pub mod assets;
#[cfg(feature = "render")]
pub mod capture;
#[cfg(feature = "render")]
pub mod card;
#[cfg(feature = "render")]
pub mod loading;
pub mod random;
pub mod record_replacement;
#[cfg(feature = "seat")]
pub mod seat;
#[cfg(feature = "render")]
pub mod token;
