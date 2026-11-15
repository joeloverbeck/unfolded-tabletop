//! A game's start of its agent seat, in either form: from "this process was asked to
//! seat an agent" to the end of the session. The game hands [`Start::run`] how to
//! build its table, with its own marker, limits and words; the order of the steps
//! and every refusal are here.
//!
//! In batch form the seat builds the table, opens and admits it, and answers standard
//! input. Held on a socket, the command an agent runs claims the socket and starts the
//! same executable again as the seat ([`socket::launch`]); that seat leaves the
//! launching shell's session, takes the listener, builds, opens and admits the table,
//! announces `ready` and serves until `quit`, the table's own exit or the idle end.
//! Every failure before `ready` prints exactly one `refused` response ([`refuse`]) as
//! the first line, which a launching command relays as its own.

use super::{Game, Seat, Unadmitted, refuse, socket};
use bevy::prelude::{App, World};
use std::{path::Path, time::Duration};

/// How long a table may take to be admitted, unless the game says otherwise:
/// asynchronous loading runs in wall-clock time.
pub const ADMISSION: Duration = Duration::from_secs(120);

/// How long a held session may wait for a request before it ends as `quit` does,
/// unless the game says otherwise.
pub const IDLE: Duration = Duration::from_secs(30 * 60);

/// What a game's words for an admission that ran out of time read: its table, and
/// the time it was allowed.
pub type Unfinished = fn(&World, Duration) -> String;

/// A game's table as the game builds it, with its windowless surface installed
/// ([`crate::capture::install`]), before the seat opens it.
pub struct Table<G, K = ()> {
    pub app: App,
    pub game: G,
    /// Whatever must outlive the app, such as a lease on the table's artwork. It is
    /// dropped after the app.
    pub kept: K,
}

impl<G> Table<G> {
    pub fn new(app: App, game: G) -> Self {
        Self {
            app,
            game,
            kept: (),
        }
    }

    /// The same table, holding `kept` until after its app has ended.
    pub fn keeping<K>(self, kept: K) -> Table<G, K> {
        Table {
            app: self.app,
            game: self.game,
            kept,
        }
    }
}

/// An admitted table with an agent seated at it. Its fields drop in order: the app,
/// then the seat, then what the table keeps.
pub struct Seated<G, K = ()> {
    pub app: App,
    pub seat: Seat<G>,
    pub kept: K,
}

/// How a game starts its seat: the environment variable that marks the seat a
/// launcher started ([`socket::detached`]), the time admission may take, how long a
/// held session may idle, and the game's words for an admission that ran out of time.
#[derive(Clone, Copy)]
pub struct Start {
    marker: &'static str,
    admission: Duration,
    idle: Duration,
    unfinished: Unfinished,
}

impl Start {
    /// The game's marker and words, with an admission limit of [`ADMISSION`] and an
    /// idle period of [`IDLE`].
    pub const fn new(marker: &'static str, unfinished: Unfinished) -> Self {
        Self {
            marker,
            admission: ADMISSION,
            idle: IDLE,
            unfinished,
        }
    }

    /// The time admission may take.
    #[must_use]
    pub const fn admission(self, limit: Duration) -> Self {
        Self {
            admission: limit,
            ..self
        }
    }

    /// How long a held session may wait for a request before it ends.
    #[must_use]
    pub const fn idle(self, period: Duration) -> Self {
        Self {
            idle: period,
            ..self
        }
    }

    /// Open a built table and admit it within the limit, in this process: what a start
    /// does before `ready`, for a game's tests as for its seat. The error is the game's
    /// refusal, or its words for an admission that ran out of time.
    pub fn seat<G: Game, K>(&self, table: Table<G, K>) -> Result<Seated<G, K>, String> {
        let Table {
            mut app,
            game,
            kept,
        } = table;
        let mut seat = Seat::open(game, &mut app)?;
        seat.admit(&mut app, self.admission)
            .map_err(|unadmitted| match unadmitted {
                Unadmitted::Refused(reason) => reason,
                Unadmitted::Unfinished => (self.unfinished)(app.world(), self.admission),
            })?;
        Ok(Seated { app, seat, kept })
    }

    /// Seat an agent at the table `build` makes, in batch form, or held on `socket`,
    /// until the session ends. Every failure before `ready` prints one `refused`
    /// response first and returns its error, for the game's standard error and
    /// unsuccessful exit.
    pub fn run<G: Game, K>(
        self,
        socket: Option<&Path>,
        build: impl FnOnce() -> Result<Table<G, K>, String>,
    ) -> Result<(), String> {
        let Some(path) = socket else {
            let mut seated = build().and_then(|table| self.seat(table)).or_else(refuse)?;
            let Seated { app, seat, .. } = &mut seated;
            return seat.serve(app, std::io::stdin().lock(), std::io::stdout().lock());
        };
        if !socket::detached(self.marker) {
            // The command an agent ran: it returns once the seat it starts is admitted.
            return socket::launch(path, self.marker).or_else(refuse);
        }
        let listener = socket::detach(path, self.marker).or_else(refuse)?;
        let mut seated = build().and_then(|table| self.seat(table)).or_else(refuse)?;
        let Seated { app, seat, .. } = &mut seated;
        socket::announce(&seat.ready(app))?;
        listener.serve(seat, app, self.idle)
    }
}
