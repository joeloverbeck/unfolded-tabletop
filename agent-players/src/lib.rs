//! What a game's agent players need that is no game's own, without Bevy. An agent player
//! is an external agent that plays at a table someone else is running; the running game's
//! side of its socket is `unfolded_tabletop::agent_transport`.
//!
//! [`client`] is the player's side: one request, one answer, and its tool's identifiers of
//! its session. [`seats`] is the table's: who holds each seat, and how the session that
//! holds one submits its choices and waits.
#![cfg(unix)]

pub mod client;
pub mod seats;
