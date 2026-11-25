//! Who holds each seat of a game, and how the session that holds one waits for what
//! needs it.
//!
//! A seat is the human's, handed over and unclaimed, or held by one session
//! ([`Holder`]). One session holds exactly one seat. [`Seats::claim`] gives it an
//! identity, which it presents on every request and keeps until it leaves, its seat is
//! taken back or the table closes; an identity is the only thing a holder is known by.
//! A session is recognised at a join, and only there, by the identity or by the tool's
//! own session identifiers it gives ([`client::session`](crate::client::session)): one
//! that holds a seat is refused another. A session that gives neither is a new one.
//!
//! **Submissions.** [`Seats::submit`] answers at once: applied, held or refused. A held
//! choice waits for the table, never for another player. [`Seats::settle`] applies it
//! once the game says the table is settled, at the revision it was made for, and
//! releases it with the reason otherwise. Taking the seat back cancels it.
//!
//! **Waiting.** A wait has no period unless the session asks for one. A session has one
//! wait: a newer one replaces it, and one whose client has died ends
//! ([`Seats::end_hung_up`]). A request that submits and then waits is a submission and
//! then a wait on the same reply: one refused for the session's own next decision is
//! answered as the submission is, at once.
//!
//! **Events.** Each is told once. A session's own decision is told by the answer to its
//! submission ([`Seats::told_in_answer`]) or by its wait ([`Seats::tell`]), whichever
//! comes first. A wait may be one nobody reads any more, whose tool was interrupted and
//! left its client running. So a decision told only by a wait is told again to the
//! session's next wait, unless the session has been answered since with a view of it
//! ([`Seats::read`]) or has submitted. Once a decision was told in an answer, a wait
//! that could tell only it is refused ([`Waiting::Refused`]) rather than held for the
//! session's own move. The end of a session is for the game to tell its held wait
//! ([`Ended`]), and each later request of its identity ([`Seats::ended`]).
//!
//! What no test here holds is a real tool: that its interrupted wait runs on, and that its
//! commands are given its identifiers. Cuba Libre #198 found both.
//!
//! Not here: what the seats are, who decides, what is hidden from whom, how a choice is
//! made and committed ([`Game`]), the requests, the answers and their words.

use std::time::{Duration, Instant};

/// The game as a submission reaches it.
pub trait Game {
    type Seat;
    type Choice;

    /// The revision of the position now. It moves with every choice applied.
    fn revision(&self) -> u64;

    /// Whether a choice applied now would show its consequences at once: nothing is in
    /// motion and nothing of the human's is open over the table.
    fn settled(&self) -> bool;

    /// Whether `seat` may make `choice` at the current revision, and if not, why.
    fn check(&self, seat: &Self::Seat, choice: &Self::Choice) -> Result<(), String>;

    /// Apply `choice` through the game's own commit. An error leaves the position and
    /// the save as they were.
    fn commit(&mut self, seat: &Self::Seat, choice: &Self::Choice) -> Result<(), String>;
}

/// Who holds a seat.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Holder {
    /// The human at the table.
    Human,
    /// Handed over, and held by no session yet.
    Unclaimed,
    /// The session with this identity.
    Session(String),
}

/// What a joining session says of itself.
#[derive(Clone, Copy, Debug, Default)]
pub struct Joining<'a> {
    /// The identity an earlier join gave it, if it still has one.
    pub identity: Option<&'a str>,
    /// Its tool's identifiers of the session, if the tool gives any.
    pub session: Option<&'a str>,
}

/// Why a claim was refused. Nothing changed.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ClaimRefused<S> {
    /// The joining session gave the identity of the session that holds this seat.
    Holds(S),
    /// The joining session's tool identifiers are those of the session that holds this
    /// seat.
    SameSession(S),
    /// Another session holds the seat.
    Held,
    /// The human has not handed the seat over.
    Humans,
}

/// What became of a submission, at once.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Submitted {
    Applied,
    /// It waits for the table to settle, in place of any choice the session had held.
    Held,
    Refused(Refusal),
}

/// Why a submission was refused, or a held choice released. Nothing changed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// No session has that identity.
    Unseated,
    /// It was made for another revision than this one.
    Stale { current: u64 },
    /// The game refused it, in its words.
    Game(String),
}

/// A choice a session made that was held and then not applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Released<C> {
    pub choice: C,
    pub why: Refusal,
}

/// What became of one held choice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settled {
    /// The identity of the session that made it.
    pub session: String,
    /// Whether it was applied. One that was not is told to the session's next wait.
    pub applied: bool,
}

/// A session that ended, with what it had in hand.
#[derive(Debug, PartialEq, Eq)]
pub struct Ended<C, R> {
    pub session: String,
    /// Its held wait, to be answered that it ended.
    pub wait: Option<R>,
    /// Its held choice, which is cancelled.
    pub choice: Option<C>,
}

/// What a wait may tell its session, as the game finds it now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event<N> {
    /// Its seat's decision at this revision.
    Decision(u64),
    /// Anything else of the game's, which the game gives only until it has been told.
    News(N),
}

/// What a wait was answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Told<C, N> {
    /// The session's held choice was released.
    Released(Released<C>),
    Decision,
    News(N),
    /// The period the session asked for ended.
    Elapsed,
}

/// What became of a wait as it arrived.
#[derive(Debug, PartialEq, Eq)]
pub enum Waiting<R> {
    /// It is held, in place of the session's older one, which is to be answered so.
    Held { replaced: Option<R> },
    /// It could tell only the decision the session was told in an answer and has not
    /// made, so nothing the session waits for would end it.
    Refused(R),
}

#[derive(Debug)]
struct Wait<R> {
    reply: R,
    deadline: Option<Instant>,
}

#[derive(Debug)]
struct HeldChoice<C> {
    revision: u64,
    choice: C,
    /// The order in which choices were held.
    order: u64,
}

/// The decision a session was last told of.
#[derive(Clone, Copy, Debug)]
struct ToldDecision {
    revision: u64,
    /// Only by a wait, which nobody may have read.
    unread: bool,
}

#[derive(Debug)]
struct Session<S, C, R> {
    id: String,
    seat: S,
    /// The tool's identifiers of it.
    tool: Option<String>,
    heard: Instant,
    told: Option<ToldDecision>,
    held: Option<HeldChoice<C>>,
    released: Option<Released<C>>,
    wait: Option<Wait<R>>,
}

/// The player seats of one game, the sessions that hold them, and what those sessions
/// have in hand. `S` is the game's seat, `C` its choice, and `R` how it answers a wait. It is
/// runtime state of the table, and no part of a save.
#[derive(Debug)]
pub struct Seats<S, C, R> {
    /// Each seat, and whether the human handed it over.
    seats: Vec<(S, bool)>,
    sessions: Vec<Session<S, C, R>>,
    /// Each ended identity and why its session ended.
    ended: Vec<(String, String)>,
    order: u64,
}

impl<S: Clone + PartialEq, C, R> Seats<S, C, R> {
    /// A game's seats, every one the human's.
    pub fn new(seats: impl IntoIterator<Item = S>) -> Self {
        Self {
            seats: seats.into_iter().map(|seat| (seat, false)).collect(),
            sessions: Vec::new(),
            ended: Vec::new(),
            order: 0,
        }
    }

    fn session(&self, id: &str) -> Option<&Session<S, C, R>> {
        self.sessions.iter().find(|session| session.id == id)
    }

    fn session_mut(&mut self, id: &str) -> Option<&mut Session<S, C, R>> {
        self.sessions.iter_mut().find(|session| session.id == id)
    }

    /// Who holds `seat`. A seat the game does not have is the human's.
    pub fn holder(&self, seat: &S) -> Holder {
        if !self.seats.contains(&(seat.clone(), true)) {
            return Holder::Human;
        }
        self.sessions
            .iter()
            .find(|session| session.seat == *seat)
            .map_or(Holder::Unclaimed, |session| {
                Holder::Session(session.id.clone())
            })
    }

    /// The seat the session `id` holds.
    pub fn seat(&self, id: &str) -> Option<&S> {
        self.session(id).map(|session| &session.seat)
    }

    /// The identities of the sessions that hold a seat, in the order they joined.
    pub fn sessions(&self) -> impl Iterator<Item = &str> {
        self.sessions.iter().map(|session| session.id.as_str())
    }

    /// Why the session that had this identity ended, for any request it still makes.
    pub fn ended(&self, id: &str) -> Option<&str> {
        self.ended
            .iter()
            .find(|(ended, _)| ended == id)
            .map(|(_, reason)| reason.as_str())
    }

    /// Hand `seat` over, for a session to claim. One already handed over stays as it is.
    pub fn hand_over(&mut self, seat: &S) {
        self.set_handed(seat, true);
    }

    /// Give `seat` back to the human. The session that held it ends at once, for `reason`.
    #[must_use = "The ended session's wait is to be answered"]
    pub fn take_back(&mut self, seat: &S, reason: &str) -> Option<Ended<C, R>> {
        self.set_handed(seat, false);
        let id = self
            .sessions
            .iter()
            .find(|session| session.seat == *seat)?
            .id
            .clone();
        self.end(&id, reason)
    }

    fn set_handed(&mut self, seat: &S, handed: bool) {
        if let Some((_, known)) = self.seats.iter_mut().find(|(known, _)| known == seat) {
            *known = handed;
        }
    }

    /// Claim `seat` for a joining session, wholly or not at all: its new identity.
    pub fn claim(
        &mut self,
        seat: &S,
        joining: Joining,
        now: Instant,
    ) -> Result<String, ClaimRefused<S>> {
        self.recognise(joining)?;
        match self.holder(seat) {
            Holder::Human => return Err(ClaimRefused::Humans),
            Holder::Session(_) => return Err(ClaimRefused::Held),
            Holder::Unclaimed => {}
        }
        let id = token();
        self.sessions.push(Session {
            id: id.clone(),
            seat: seat.clone(),
            tool: joining.session.map(str::to_owned),
            heard: now,
            told: None,
            held: None,
            released: None,
            wait: None,
        });
        Ok(id)
    }

    /// Whether a joining session already holds a seat, whichever it would claim: an
    /// error if it does, as [`claim`](Self::claim) gives.
    pub fn recognise(&self, joining: Joining) -> Result<(), ClaimRefused<S>> {
        if let Some(seat) = joining.identity.and_then(|id| self.seat(id)) {
            return Err(ClaimRefused::Holds(seat.clone()));
        }
        let same = joining.session.and_then(|tool| {
            self.sessions
                .iter()
                .find(|session| session.tool.as_deref() == Some(tool))
        });
        match same {
            Some(session) => Err(ClaimRefused::SameSession(session.seat.clone())),
            None => Ok(()),
        }
    }

    /// The session `id` leaves; its seat stays handed over. Its identity is then unknown.
    #[must_use = "The session's held wait is to be answered"]
    pub fn leave(&mut self, id: &str) -> Option<Ended<C, R>> {
        self.remove(id)
    }

    /// End every session, for `reason`: the table closes or starts again. The seats stay
    /// handed over.
    #[must_use = "The ended sessions' waits are to be answered"]
    pub fn close(&mut self, reason: &str) -> Vec<Ended<C, R>> {
        let ids: Vec<String> = self.sessions().map(str::to_owned).collect();
        ids.iter().filter_map(|id| self.end(id, reason)).collect()
    }

    fn end(&mut self, id: &str, reason: &str) -> Option<Ended<C, R>> {
        let ended = self.remove(id)?;
        self.ended.push((id.to_owned(), reason.to_owned()));
        Some(ended)
    }

    fn remove(&mut self, id: &str) -> Option<Ended<C, R>> {
        let index = self.sessions.iter().position(|session| session.id == id)?;
        let session = self.sessions.remove(index);
        Some(Ended {
            session: session.id,
            wait: session.wait.map(|wait| wait.reply),
            choice: session.held.map(|held| held.choice),
        })
    }

    /// The session `id` sent something.
    pub fn heard(&mut self, id: &str, now: Instant) {
        if let Some(session) = self.session_mut(id) {
            session.heard = now;
        }
    }

    /// How long since the session that holds `seat` was last heard from: zero while it
    /// waits, and while its choice is held, when nothing is asked of it. `None` when no
    /// session holds it.
    pub fn silence(&self, seat: &S, now: Instant) -> Option<Duration> {
        let session = self.sessions.iter().find(|session| session.seat == *seat)?;
        Some(if session.wait.is_some() || session.held.is_some() {
            Duration::ZERO
        } else {
            now.saturating_duration_since(session.heard)
        })
    }

    /// The session `id` submits `choice`, made at `revision`. Refused, it changes
    /// nothing, a choice held earlier included.
    pub fn submit(
        &mut self,
        id: &str,
        revision: u64,
        choice: C,
        game: &mut impl Game<Seat = S, Choice = C>,
    ) -> Submitted {
        let Some(seat) = self.seat(id).cloned() else {
            return Submitted::Refused(Refusal::Unseated);
        };
        if let Err(refusal) = check(game, revision, &seat, &choice) {
            return Submitted::Refused(refusal);
        }
        if game.settled() {
            if let Err(reason) = game.commit(&seat, &choice) {
                return Submitted::Refused(Refusal::Game(reason));
            }
            self.decided(id);
            let _ = self.withdraw(id);
            return Submitted::Applied;
        }
        self.decided(id);
        let _ = self.withdraw(id);
        self.order += 1;
        let order = self.order;
        self.session_mut(id).expect("Seated").held = Some(HeldChoice {
            revision,
            choice,
            order,
        });
        Submitted::Held
    }

    /// The session `id` answered its decision in a way of the game's own, as
    /// [`submit`](Self::submit) says for itself: the decision it was told is over, so one
    /// that comes to it again is told again.
    pub fn decided(&mut self, id: &str) {
        if let Some(session) = self.session_mut(id) {
            session.told = None;
        }
    }

    /// Cancel the choice the session `id` has held, and the news of one released before.
    /// The cancelled choice.
    pub fn withdraw(&mut self, id: &str) -> Option<C> {
        let session = self.session_mut(id)?;
        session.released = None;
        session.held.take().map(|held| held.choice)
    }

    /// The choice the session `id` has held, and the revision it was made at.
    pub fn held(&self, id: &str) -> Option<(u64, &C)> {
        let held = self.session(id)?.held.as_ref()?;
        Some((held.revision, &held.choice))
    }

    /// Whether [`settle`](Self::settle) has something to do at this revision, with the
    /// table `settled` or not.
    pub fn settle_due(&self, revision: u64, settled: bool) -> bool {
        self.sessions
            .iter()
            .filter_map(|session| session.held.as_ref())
            .any(|held| settled || held.revision != revision)
    }

    /// Apply the held choices whose moment has come, in the order they were held, and
    /// release those the position has passed or the game now refuses.
    pub fn settle(&mut self, game: &mut impl Game<Seat = S, Choice = C>) -> Vec<Settled> {
        let mut holding: Vec<(u64, String)> = self
            .sessions
            .iter()
            .filter_map(|session| Some((session.held.as_ref()?.order, session.id.clone())))
            .collect();
        holding.sort_unstable();
        let mut settled = Vec::new();
        for (_, id) in holding {
            let session = self.session_mut(&id).expect("No session ends here");
            let held = session.held.as_ref().expect("Listed as holding");
            if held.revision == game.revision() && !game.settled() {
                continue;
            }
            let result = check(game, held.revision, &session.seat, &held.choice).and_then(|()| {
                game.commit(&session.seat, &held.choice)
                    .map_err(Refusal::Game)
            });
            let held = session.held.take().expect("Listed as holding");
            let applied = result.is_ok();
            if let Err(why) = result {
                session.told = None;
                session.released = Some(Released {
                    choice: held.choice,
                    why,
                });
            }
            settled.push(Settled {
                session: id,
                applied,
            });
        }
        settled
    }

    /// Hold a wait of the session `id`, which must hold a seat. `decision` is the
    /// revision of the decision it would tell, when its seat has one. `period` is the
    /// one the session asked for; one beyond what the clock can count is none.
    pub fn wait(
        &mut self,
        id: &str,
        reply: R,
        period: Option<Duration>,
        decision: Option<u64>,
        now: Instant,
    ) -> Waiting<R> {
        let Some(session) = self.session_mut(id) else {
            return Waiting::Refused(reply);
        };
        session.heard = now;
        match session.told {
            Some(told) if told.unread => session.told = None,
            Some(told)
                if decision == Some(told.revision)
                    && session.held.is_none()
                    && session.released.is_none() =>
            {
                return Waiting::Refused(reply);
            }
            _ => {}
        }
        let wait = Wait {
            reply,
            deadline: period.and_then(|period| now.checked_add(period)),
        };
        Waiting::Held {
            replaced: session.wait.replace(wait).map(|older| older.reply),
        }
    }

    /// Whether the session `id` waits.
    pub fn waits(&self, id: &str) -> bool {
        self.session(id)
            .is_some_and(|session| session.wait.is_some())
    }

    /// Answer the wait of the session `id` if its moment has come: with a choice of its
    /// own that was released, the first of `events` it has not been told, or the end of
    /// its period. `events` are the game's, in the order to tell them.
    pub fn tell<N>(
        &mut self,
        id: &str,
        events: impl IntoIterator<Item = Event<N>>,
        now: Instant,
    ) -> Option<(R, Told<C, N>)> {
        let session = self.session_mut(id)?;
        let mut events: Vec<Event<N>> = events.into_iter().collect();
        let pending = events.iter().find_map(|event| match event {
            Event::Decision(revision) => Some(*revision),
            Event::News(_) => None,
        });
        let deadline = session.wait.as_ref()?.deadline;
        let told = if let Some(released) = session.released.take() {
            Told::Released(released)
        } else if let Some(first) = events.iter().position(|event| session.untold(event)) {
            match events.swap_remove(first) {
                Event::Decision(_) => Told::Decision,
                Event::News(news) => Told::News(news),
            }
        } else if deadline.is_some_and(|deadline| now >= deadline) {
            Told::Elapsed
        } else {
            return None;
        };
        if !matches!(told, Told::Elapsed)
            && let Some(revision) = pending
            && session.untold::<N>(&Event::Decision(revision))
        {
            session.told = Some(ToldDecision {
                revision,
                unread: true,
            });
        }
        session.heard = now;
        let wait = session.wait.take().expect("Found above");
        Some((wait.reply, told))
    }

    /// Whether [`tell`](Self::tell) would answer the wait of the session `id`.
    pub fn would_tell<N>(
        &self,
        id: &str,
        events: impl IntoIterator<Item = Event<N>>,
        now: Instant,
    ) -> bool {
        let Some(session) = self.session(id) else {
            return false;
        };
        let Some(wait) = &session.wait else {
            return false;
        };
        session.released.is_some()
            || wait.deadline.is_some_and(|deadline| now >= deadline)
            || events.into_iter().any(|event| session.untold(&event))
    }

    /// The answer to a submission of the session `id` shows it its own decision at
    /// `revision`.
    pub fn told_in_answer(&mut self, id: &str, revision: u64) {
        if let Some(session) = self.session_mut(id) {
            session.told = Some(ToldDecision {
                revision,
                unread: false,
            });
        }
    }

    /// The session `id` is answered something other than a wait that shows its position
    /// whole, so what its last wait told it is known.
    pub fn read(&mut self, id: &str) {
        if let Some(told) = self
            .session_mut(id)
            .and_then(|session| session.told.as_mut())
        {
            told.unread = false;
        }
    }

    /// Whether a held wait's client has died, as `hung_up` says of its reply.
    pub fn hung_up(&self, hung_up: impl Fn(&R) -> bool) -> bool {
        self.sessions
            .iter()
            .filter_map(|session| session.wait.as_ref())
            .any(|wait| hung_up(&wait.reply))
    }

    /// End each held wait whose client has died, before anything is told to it: its
    /// session was last heard from now, and what its wait would have told stays to be
    /// told. Its seat and its held choice stay as they are. The sessions' identities.
    pub fn end_hung_up(&mut self, hung_up: impl Fn(&R) -> bool, now: Instant) -> Vec<String> {
        let mut ended = Vec::new();
        for session in &mut self.sessions {
            if session.wait.take_if(|wait| hung_up(&wait.reply)).is_some() {
                session.heard = now;
                ended.push(session.id.clone());
            }
        }
        ended
    }
}

impl<S, C, R> Session<S, C, R> {
    /// Whether `event` is still to be told. A decision is not while the session's choice
    /// in it is held.
    fn untold<N>(&self, event: &Event<N>) -> bool {
        match event {
            Event::Decision(revision) => {
                self.told.is_none_or(|told| told.revision != *revision)
                    && self
                        .held
                        .as_ref()
                        .is_none_or(|held| held.revision != *revision)
            }
            Event::News(_) => true,
        }
    }
}

fn check<G: Game>(
    game: &G,
    revision: u64,
    seat: &G::Seat,
    choice: &G::Choice,
) -> Result<(), Refusal> {
    let current = game.revision();
    if revision != current {
        return Err(Refusal::Stale { current });
    }
    game.check(seat, choice).map_err(Refusal::Game)
}

/// An identity no earlier session of this process's tables can have had.
fn token() -> String {
    use std::io::Read;
    let mut bytes = [0u8; 6];
    let random =
        std::fs::File::open("/dev/urandom").and_then(|mut file| file.read_exact(&mut bytes));
    if random.is_err() {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_nanos());
        bytes.copy_from_slice(&nanos.to_le_bytes()[..6]);
    }
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
