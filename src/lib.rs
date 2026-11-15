//! Concrete mechanisms shared by named consumer games: presentation, screen
//! capture and the agent seat, behind `render` and `seat`.
//!
//! Seeded randomness, hover, recoverable file replacement, asset reads and the
//! held-seat client live in packages of their own that cannot depend on Bevy.
//! The re-exports below keep their earlier `unfolded_tabletop::…` paths working
//! while the games move to those packages.
pub use unfolded_assets as assets;
#[cfg(feature = "render")]
pub mod camera;
#[cfg(feature = "render")]
pub mod capture;
#[cfg(feature = "render")]
pub mod card;
#[cfg(feature = "render")]
pub mod count_label;
pub use unfolded_hover as hover;
#[cfg(feature = "render")]
pub mod loading;
#[cfg(feature = "render")]
pub mod printed;
pub use unfolded_random as random;
pub use unfolded_record_replacement as record_replacement;
#[cfg(feature = "seat")]
pub mod seat;
#[cfg(unix)]
pub use unfolded_seat_client as seat_client;
#[cfg(feature = "render")]
pub mod token;
