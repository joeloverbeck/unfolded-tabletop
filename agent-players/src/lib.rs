//! What a game's agent players need that is no game's own, without Bevy. An agent player
//! is an external agent that plays at a table someone else is running; the running game's
//! side of its socket is `unfolded_tabletop::agent_transport`.
//!
//! [`client`] is the player's side: one request, one answer.
#![cfg(unix)]

pub mod client;
