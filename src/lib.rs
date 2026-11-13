//! Concrete mechanisms shared by named consumer games: presentation, screen
//! capture, and seeded randomness.
pub mod assets;
#[cfg(feature = "render")]
pub mod capture;
#[cfg(feature = "render")]
pub mod card;
#[cfg(feature = "render")]
pub mod loading;
pub mod random;
#[cfg(feature = "render")]
pub mod token;
