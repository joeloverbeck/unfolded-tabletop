//! Seam: who holds a seat and how its session waits, against a stand-in game with no
//! window, no socket and no model. Expectations come from Cuba Libre #197 ("The shared
//! mechanism") and #201, with its corrections 17 to 19.

use std::time::{Duration, Instant};
use unfolded_agent_players::seats::{
    Ended, Event, Game, Holder, Joining, Refusal, Released, Seats, Settled, Submitted, Told,
    Unclaimed, Waiting,
};

const RED: &str = "red";
const BLUE: &str = "blue";
const GREEN: &str = "green";

/// A position each choice moves on by one, for the seat whose turn it is, and the save
/// each commit writes.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Position {
    revision: u64,
    turn: &'static str,
    settled: bool,
    save: Vec<String>,
    /// The commit fails, as a save that cannot be written does.
    broken: bool,
}

impl Game for Position {
    type Seat = &'static str;
    type Choice = &'static str;

    fn revision(&self) -> u64 {
        self.revision
    }

    fn settled(&self) -> bool {
        self.settled
    }

    fn check(&self, seat: &&'static str, choice: &&'static str) -> Result<(), String> {
        if *seat != self.turn {
            Err(format!("It is {}'s turn.", self.turn))
        } else if *choice == "impossible" {
            Err("That is not offered.".into())
        } else {
            Ok(())
        }
    }

    fn commit(&mut self, seat: &&'static str, choice: &&'static str) -> Result<(), String> {
        if self.broken {
            return Err("The save could not be written.".into());
        }
        self.revision += 1;
        self.save.push(format!("{seat}: {choice}"));
        Ok(())
    }
}

type Table = Seats<&'static str, &'static str, u32>;

fn position() -> Position {
    Position {
        revision: 7,
        turn: RED,
        settled: true,
        save: vec!["the opening".into()],
        broken: false,
    }
}

/// Red and blue handed over, green the human's.
fn table() -> Table {
    let mut seats = Seats::new([RED, BLUE, GREEN]);
    seats.hand_over(&RED);
    seats.hand_over(&BLUE);
    seats
}

/// Join as a session that says nothing of itself.
fn join(seats: &mut Table, seat: &'static str, now: Instant) -> String {
    seats.claim(&seat, Joining::default(), now).unwrap()
}

fn wait(seats: &mut Table, id: &str, reply: u32, decision: Option<u64>, now: Instant) {
    assert_eq!(
        seats.wait(id, reply, None, decision, now),
        Waiting::Held { replaced: None }
    );
}

/// What a wait is told when the game has only a decision for it.
fn decision(
    seats: &mut Table,
    id: &str,
    revision: u64,
    now: Instant,
) -> Option<(u32, Told<&'static str, &'static str>)> {
    seats.tell(id, [Event::Decision(revision)], now)
}

fn news(
    seats: &mut Table,
    id: &str,
    news: &'static str,
    now: Instant,
) -> Option<(u32, Told<&'static str, &'static str>)> {
    seats.tell(id, [Event::News(news)], now)
}

fn nothing(
    seats: &mut Table,
    id: &str,
    now: Instant,
) -> Option<(u32, Told<&'static str, &'static str>)> {
    seats.tell(id, [], now)
}

#[test]
fn a_seat_is_the_humans_handed_over_or_held() {
    let now = Instant::now();
    let mut seats = Table::new([RED, BLUE]);
    assert_eq!(seats.holder(&RED), Holder::Human);
    seats.hand_over(&RED);
    assert_eq!(seats.holder(&RED), Holder::Unclaimed);
    let red = join(&mut seats, RED, now);
    assert_eq!(seats.holder(&RED), Holder::Session(red.clone()));
    assert_eq!(seats.seat(&red), Some(&RED));
    assert_eq!(seats.holder(&BLUE), Holder::Human);
    assert_eq!(seats.holder(&"no such seat"), Holder::Human);
    // Handing over a seat that is held changes nothing.
    seats.hand_over(&RED);
    assert_eq!(seats.holder(&RED), Holder::Session(red.clone()));
    // A session that leaves gives its seat to whoever joins next, and its identity is
    // not used again.
    assert_eq!(
        seats.leave(&red),
        Some(Ended {
            session: red.clone(),
            wait: None,
            choice: None
        })
    );
    assert_eq!(seats.holder(&RED), Holder::Unclaimed);
    assert_eq!((seats.seat(&red), seats.ended(&red)), (None, None));
    assert_ne!(join(&mut seats, RED, now), red);
}

#[test]
fn a_claim_is_refused_with_nothing_changed() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    let before = format!("{seats:?}");
    // Two sessions claim one seat: the second is refused.
    assert_eq!(
        seats.claim(&RED, Joining::default(), now),
        Err(Unclaimed::Held)
    );
    // A seat not handed over, and one the game does not have.
    assert_eq!(
        seats.claim(&GREEN, Joining::default(), now),
        Err(Unclaimed::Humans)
    );
    assert_eq!(
        seats.claim(&"no such seat", Joining::default(), now),
        Err(Unclaimed::Humans)
    );
    // The holder's identity, whichever seat it claims.
    for seat in [BLUE, RED, GREEN] {
        assert_eq!(
            seats.claim(
                &seat,
                Joining {
                    identity: Some(&red),
                    session: None
                },
                now
            ),
            Err(Unclaimed::Holds(RED))
        );
    }
    assert_eq!(format!("{seats:?}"), before);
}

#[test]
fn one_session_holds_one_seat_by_its_tools_identifiers() {
    let now = Instant::now();
    let mut seats = table();
    let joining = |session| Joining {
        identity: None,
        session: Some(session),
    };
    let red = seats.claim(&RED, joining("thread 1"), now).unwrap();
    let before = format!("{seats:?}");
    assert_eq!(
        seats.claim(&BLUE, joining("thread 1"), now),
        Err(Unclaimed::SameSession(RED))
    );
    assert_eq!(format!("{seats:?}"), before);
    // It is recognised at a join and nowhere else: the identity is what holds the seat.
    assert_eq!(seats.seat("thread 1"), None);
    assert!(seats.claim(&BLUE, joining("thread 2"), now).is_ok());
    // Once it holds no seat it may join again.
    let _ = seats.take_back(&RED, "Taken back.");
    seats.hand_over(&RED);
    let again = seats
        .claim(
            &RED,
            Joining {
                identity: Some(&red),
                session: Some("thread 1"),
            },
            now,
        )
        .unwrap();
    assert_ne!(again, red);
}

/// A tool that gives no identifiers: only the identity the session gives is recognised.
#[test]
fn a_session_that_says_nothing_of_itself_is_a_new_one() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    assert_ne!(red, blue);
}

#[test]
fn a_submission_is_applied_at_once_at_a_settled_table() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    assert_eq!(
        seats.submit(&red, 7, "march", &mut game),
        Submitted::Applied
    );
    assert_eq!(game.revision, 8);
    assert_eq!(game.save, ["the opening", "red: march"]);
    assert_eq!(seats.held(&red), None);
}

/// A choice by a session without a seat, for another seat's decision, at a stale or a
/// future revision, not offered, or that cannot be saved, at a settled table and at one
/// that is not.
#[test]
fn a_refused_submission_leaves_the_position_the_save_and_every_seat_unchanged() {
    let now = Instant::now();
    for settled in [true, false] {
        let (mut seats, mut game) = (table(), position());
        let red = join(&mut seats, RED, now);
        let blue = join(&mut seats, BLUE, now);
        wait(&mut seats, &blue, 1, None, now);
        game.settled = settled;
        let (position, sessions) = (game.clone(), format!("{seats:?}"));
        for (id, revision, choice, refusal) in [
            ("nobody", 7, "march", Refusal::Unseated),
            (&blue, 7, "march", Refusal::Game("It is red's turn.".into())),
            (&red, 6, "march", Refusal::Stale { current: 7 }),
            (&red, 8, "march", Refusal::Stale { current: 7 }),
            (
                &red,
                7,
                "impossible",
                Refusal::Game("That is not offered.".into()),
            ),
        ] {
            assert_eq!(
                seats.submit(id, revision, choice, &mut game),
                Submitted::Refused(refusal)
            );
        }
        assert_eq!(game, position);
        assert_eq!(format!("{seats:?}"), sessions);
    }
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    game.broken = true;
    let before = game.clone();
    assert_eq!(
        seats.submit(&red, 7, "march", &mut game),
        Submitted::Refused(Refusal::Game("The save could not be written.".into()))
    );
    assert_eq!((game, seats.held(&red)), (before, None));
}

#[test]
fn a_choice_at_an_unsettled_table_is_held_until_it_settles() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    game.settled = false;
    let before = game.clone();
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    assert!(!seats.settle_due(7, false));
    assert_eq!(seats.settle(&mut game), []);
    assert_eq!(game, before);
    assert_eq!(seats.held(&red), Some((7, &"march")));
    game.settled = true;
    assert!(seats.settle_due(7, true));
    assert_eq!(
        seats.settle(&mut game),
        [Settled {
            session: red.clone(),
            applied: true
        }]
    );
    assert_eq!(game.save, ["the opening", "red: march"]);
    assert_eq!(seats.held(&red), None);
    // Nothing is told of it: it needs nothing of the session.
    wait(&mut seats, &red, 1, None, now);
    assert_eq!(nothing(&mut seats, &red, now), None);
}

#[test]
fn a_later_choice_replaces_the_held_one_and_a_refused_one_does_not() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    game.settled = false;
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    assert!(matches!(
        seats.submit(&red, 7, "impossible", &mut game),
        Submitted::Refused(_)
    ));
    assert_eq!(seats.held(&red), Some((7, &"march")));
    assert_eq!(seats.submit(&red, 7, "rally", &mut game), Submitted::Held);
    game.settled = true;
    assert_eq!(seats.submit(&red, 7, "pass", &mut game), Submitted::Applied);
    assert_eq!(seats.settle(&mut game), []);
    assert_eq!(game.save, ["the opening", "red: pass"]);
}

/// A held choice is released, with the reason, when the position has moved, settled or
/// not, and when the game refuses it as it is applied.
#[test]
fn a_held_choice_is_released_with_the_reason() {
    let now = Instant::now();
    let held = |now| {
        let (mut seats, mut game) = (table(), position());
        let red = join(&mut seats, RED, now);
        game.settled = false;
        assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
        wait(&mut seats, &red, 1, None, now);
        (seats, game, red)
    };

    let (mut seats, mut game, red) = held(now);
    game.revision = 8;
    assert!(seats.settle_due(8, false));
    assert_eq!(
        seats.settle(&mut game),
        [Settled {
            session: red.clone(),
            applied: false
        }]
    );
    assert_eq!(game.save, ["the opening"]);
    assert!(seats.would_tell::<()>(&red, [], now));
    assert_eq!(
        nothing(&mut seats, &red, now),
        Some((
            1,
            Told::Released(Released {
                choice: "march",
                why: Refusal::Stale { current: 8 }
            })
        ))
    );
    // It is told once.
    wait(&mut seats, &red, 2, None, now);
    assert_eq!(nothing(&mut seats, &red, now), None);

    for (change, why) in [
        (
            (|game| game.turn = BLUE) as fn(&mut Position),
            "It is blue's turn.",
        ),
        (|game| game.broken = true, "The save could not be written."),
    ] {
        let (mut seats, mut game, red) = held(now);
        change(&mut game);
        game.settled = true;
        let before = game.clone();
        assert_eq!(seats.settle(&mut game).len(), 1);
        assert_eq!(game, before);
        // The decision is the session's again, and it is told of it with the release.
        assert_eq!(
            decision(&mut seats, &red, 7, now),
            Some((
                1,
                Told::Released(Released {
                    choice: "march",
                    why: Refusal::Game(why.into())
                })
            ))
        );
        wait(&mut seats, &red, 2, Some(7), now);
        assert_eq!(
            decision(&mut seats, &red, 7, now),
            Some((2, Told::Decision))
        );
    }
}

/// Two choices held for one position: the first in time is applied, and the other is
/// released.
#[test]
fn held_choices_are_applied_in_the_order_they_were_held() {
    let now = Instant::now();
    struct Open(Position);
    impl Game for Open {
        type Seat = &'static str;
        type Choice = &'static str;
        fn revision(&self) -> u64 {
            self.0.revision
        }
        fn settled(&self) -> bool {
            self.0.settled
        }
        fn check(&self, _: &&'static str, _: &&'static str) -> Result<(), String> {
            Ok(())
        }
        fn commit(&mut self, seat: &&'static str, choice: &&'static str) -> Result<(), String> {
            self.0.commit(seat, choice)
        }
    }
    let (mut seats, mut game) = (table(), Open(position()));
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    game.0.settled = false;
    assert_eq!(seats.submit(&blue, 7, "strike", &mut game), Submitted::Held);
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    game.0.settled = true;
    assert_eq!(
        seats.settle(&mut game),
        [
            Settled {
                session: blue,
                applied: true
            },
            Settled {
                session: red,
                applied: false
            }
        ]
    );
    assert_eq!(game.0.save, ["the opening", "blue: strike"]);
}

/// Taking back before a choice, with one held, and with the session waiting.
#[test]
fn taking_a_seat_back_ends_its_session_and_cancels_its_held_choice() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    wait(&mut seats, &blue, 1, None, now);
    let other = format!("{:?}", seats.holder(&BLUE));
    game.settled = false;
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    wait(&mut seats, &red, 2, None, now);
    assert_eq!(
        seats.take_back(&RED, "The table took red back."),
        Some(Ended {
            session: red.clone(),
            wait: Some(2),
            choice: Some("march")
        })
    );
    assert_eq!(seats.holder(&RED), Holder::Human);
    game.settled = true;
    assert_eq!(seats.settle(&mut game), []);
    assert_eq!(game.save, ["the opening"]);
    // Told again to whatever the session still sends, in case nobody read it.
    assert_eq!(seats.ended(&red), Some("The table took red back."));
    assert_eq!(
        seats.submit(&red, 7, "march", &mut game),
        Submitted::Refused(Refusal::Unseated)
    );
    // The other session is as it was.
    assert_eq!(format!("{:?}", seats.holder(&BLUE)), other);
    assert!(seats.waits(&blue));
    assert_eq!(seats.ended(&blue), None);
    // Taking back a seat nobody holds ends nothing.
    assert_eq!(seats.take_back(&RED, "Again."), None);
}

/// The table closing answers each wait once, and nothing is left to answer.
#[test]
fn closing_ends_each_session_once() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    wait(&mut seats, &blue, 1, None, now);
    assert_eq!(
        seats.close("The table closed."),
        [
            Ended {
                session: red.clone(),
                wait: None,
                choice: None
            },
            Ended {
                session: blue,
                wait: Some(1),
                choice: None
            }
        ]
    );
    assert_eq!(seats.close("The table closed."), []);
    assert_eq!(seats.ended(&red), Some("The table closed."));
    // The seats are still handed over, as when a table starts again.
    assert_eq!(seats.holder(&RED), Holder::Unclaimed);
    join(&mut seats, RED, now);
}

#[test]
fn a_wait_has_no_period_unless_the_session_asks_for_one() {
    let now = Instant::now();
    let year = now + Duration::from_secs(365 * 24 * 3600);
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    wait(&mut seats, &red, 1, None, now);
    assert!(!seats.would_tell::<()>(&red, [], year));
    assert_eq!(nothing(&mut seats, &red, year), None);

    let minute = Duration::from_secs(60);
    assert_eq!(
        seats.wait(&red, 2, Some(minute), None, now),
        Waiting::Held { replaced: Some(1) }
    );
    let before = now + minute - Duration::from_millis(1);
    assert!(!seats.would_tell::<()>(&red, [], before));
    assert_eq!(nothing(&mut seats, &red, before), None);
    assert!(seats.would_tell::<()>(&red, [], now + minute));
    assert_eq!(
        nothing(&mut seats, &red, now + minute),
        Some((2, Told::Elapsed))
    );
    assert!(!seats.waits(&red));

    // A period beyond what the clock can count is none.
    assert_eq!(
        seats.wait(&red, 3, Some(Duration::MAX), None, now),
        Waiting::Held { replaced: None }
    );
    assert_eq!(nothing(&mut seats, &red, year), None);
}

#[test]
fn a_decision_told_by_the_answer_to_a_submission_is_not_told_by_a_wait() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    // A wait of the session's is held meanwhile, as a background one is.
    wait(&mut seats, &red, 1, Some(7), now);
    assert_eq!(
        seats.submit(&red, 7, "march", &mut game),
        Submitted::Applied
    );
    seats.told_in_answer(&red, 8);
    assert!(!seats.would_tell::<()>(&red, [Event::Decision(8)], now));
    assert_eq!(decision(&mut seats, &red, 8, now), None);
    // Another wait could tell only the same: it is refused, and not held for a move of
    // the session's own. The first stays.
    assert_eq!(seats.wait(&red, 2, None, Some(8), now), Waiting::Refused(2));
    assert!(seats.waits(&red));
    // Something else still ends the wait, and a decision after this one is told.
    assert_eq!(
        news(&mut seats, &red, "a message", now),
        Some((1, Told::News("a message")))
    );
    assert_eq!(
        seats.submit(&red, 8, "rally", &mut game),
        Submitted::Applied
    );
    wait(&mut seats, &red, 3, None, now);
    assert_eq!(nothing(&mut seats, &red, now), None);
    assert_eq!(
        decision(&mut seats, &red, 11, now),
        Some((3, Told::Decision))
    );
}

/// Correction 17: an interrupted tool can leave a wait running that nobody reads.
#[test]
fn a_decision_told_only_to_a_wait_is_told_again_to_the_sessions_next_wait() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    wait(&mut seats, &red, 1, Some(7), now);
    assert_eq!(
        decision(&mut seats, &red, 7, now),
        Some((1, Told::Decision))
    );
    wait(&mut seats, &red, 2, Some(7), now);
    assert_eq!(
        decision(&mut seats, &red, 7, now),
        Some((2, Told::Decision))
    );
    // Any other request that shows the session the position settles that it knows.
    seats.read(&red);
    assert_eq!(seats.wait(&red, 3, None, Some(7), now), Waiting::Refused(3));
    // As does its own submission, after which a wait is for what comes next.
    assert_eq!(
        seats.submit(&red, 7, "march", &mut game),
        Submitted::Applied
    );
    wait(&mut seats, &red, 4, None, now);
    assert_eq!(nothing(&mut seats, &red, now), None);
}

/// A wait that tells something else shows the position, and so the decision, but nobody
/// may have read it.
#[test]
fn a_decision_that_came_with_other_news_is_still_told_to_the_next_wait() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    wait(&mut seats, &red, 1, Some(7), now);
    assert_eq!(
        seats.tell(&red, [Event::News("a message"), Event::Decision(7)], now),
        Some((1, Told::News("a message")))
    );
    wait(&mut seats, &red, 2, Some(7), now);
    assert_eq!(
        decision(&mut seats, &red, 7, now),
        Some((2, Told::Decision))
    );
}

/// The first thing the game lists is told, and a decision already told does not hide
/// what follows it.
#[test]
fn events_are_told_in_the_games_order() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    wait(&mut seats, &red, 1, Some(7), now);
    assert_eq!(
        seats.tell(&red, [Event::Decision(7), Event::News("a message")], now),
        Some((1, Told::Decision))
    );
    seats.read(&red);
    assert_eq!(
        seats.wait(&red, 2, None, None, now),
        Waiting::Held { replaced: None }
    );
    assert_eq!(
        seats.tell(&red, [Event::Decision(7), Event::News("a message")], now),
        Some((2, Told::News("a message")))
    );
}

/// While a session's choice is held, the decision it answers is not told to it again.
#[test]
fn a_decision_whose_choice_is_held_is_not_told() {
    let now = Instant::now();
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    game.settled = false;
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    wait(&mut seats, &red, 1, Some(7), now);
    assert_eq!(decision(&mut seats, &red, 7, now), None);
}

/// Correction 17: a session has one wait.
#[test]
fn a_newer_wait_replaces_the_older() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    wait(&mut seats, &red, 1, None, now);
    wait(&mut seats, &blue, 2, None, now);
    assert_eq!(
        seats.wait(&red, 3, None, None, now),
        Waiting::Held { replaced: Some(1) }
    );
    assert_eq!(
        decision(&mut seats, &red, 7, now),
        Some((3, Told::Decision))
    );
    assert!(seats.waits(&blue));
}

/// Corrections 11 and 19: a killed wait neither holds its seat as thinking nor swallows
/// its event.
#[test]
fn a_wait_whose_client_died_ends() {
    let now = Instant::now();
    let later = now + Duration::from_secs(900);
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    let blue = join(&mut seats, BLUE, now);
    game.settled = false;
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    wait(&mut seats, &red, 1, None, now);
    wait(&mut seats, &blue, 2, None, now);
    assert_eq!(seats.silence(&RED, later), Some(Duration::ZERO));

    let dead = |reply: &u32| *reply == 1;
    assert!(seats.hung_up(dead));
    assert_eq!(seats.end_hung_up(dead, later), [red.clone()]);
    assert!(!seats.hung_up(dead));
    assert!(!seats.waits(&red));
    assert!(seats.waits(&blue));
    // Last heard from as its client died.
    assert_eq!(
        seats.silence(&RED, later + Duration::from_secs(60)),
        Some(Duration::from_secs(60))
    );
    // Its seat and its held choice are still its own.
    assert_eq!(seats.holder(&RED), Holder::Session(red.clone()));
    game.settled = true;
    assert_eq!(seats.settle(&mut game).len(), 1);
    assert_eq!(game.save, ["the opening", "red: march"]);
    // And what its wait would have told it is told to the next.
    assert_eq!(decision(&mut seats, &red, 9, later), None);
    wait(&mut seats, &red, 3, Some(9), later);
    assert_eq!(
        decision(&mut seats, &red, 9, later),
        Some((3, Told::Decision))
    );
}

#[test]
fn silence_is_counted_from_the_last_request_or_the_last_answer_to_a_wait() {
    let now = Instant::now();
    let after = |seconds| now + Duration::from_secs(seconds);
    let mut seats = table();
    assert_eq!(seats.silence(&RED, now), None);
    let red = join(&mut seats, RED, now);
    assert_eq!(
        seats.silence(&RED, after(90)),
        Some(Duration::from_secs(90))
    );
    seats.heard(&red, after(100));
    assert_eq!(
        seats.silence(&RED, after(130)),
        Some(Duration::from_secs(30))
    );
    wait(&mut seats, &red, 1, None, after(130));
    assert_eq!(seats.silence(&RED, after(1000)), Some(Duration::ZERO));
    assert!(decision(&mut seats, &red, 7, after(1000)).is_some());
    assert_eq!(
        seats.silence(&RED, after(1010)),
        Some(Duration::from_secs(10))
    );
}

/// Nothing is asked of a session whose choice waits for the table.
#[test]
fn a_session_whose_choice_is_held_is_not_silent() {
    let now = Instant::now();
    let later = now + Duration::from_secs(300);
    let (mut seats, mut game) = (table(), position());
    let red = join(&mut seats, RED, now);
    game.settled = false;
    assert_eq!(seats.submit(&red, 7, "march", &mut game), Submitted::Held);
    assert_eq!(seats.silence(&RED, later), Some(Duration::ZERO));
    // A choice the game takes another way is withdrawn, and not applied here.
    assert_eq!(seats.withdraw(&red), Some("march"));
    assert_eq!(seats.silence(&RED, later), Some(Duration::from_secs(300)));
    game.settled = true;
    assert_eq!(seats.settle(&mut game), []);
    assert_eq!(game.save, ["the opening"]);
}

/// A game's own way of answering a decision ends it as a submission does.
#[test]
fn a_decision_that_comes_again_after_it_was_answered_is_told_again() {
    let now = Instant::now();
    let mut seats = table();
    let red = join(&mut seats, RED, now);
    wait(&mut seats, &red, 1, Some(7), now);
    assert!(decision(&mut seats, &red, 7, now).is_some());
    seats.decided(&red);
    wait(&mut seats, &red, 2, None, now);
    assert_eq!(nothing(&mut seats, &red, now), None);
    // Another player's change at the same revision asks for it again.
    assert_eq!(
        decision(&mut seats, &red, 7, now),
        Some((2, Told::Decision))
    );
}

/// A wait by an identity that holds no seat is not held.
#[test]
fn a_session_without_a_seat_has_no_wait() {
    let now = Instant::now();
    let mut seats = table();
    assert_eq!(
        seats.wait("nobody", 1, None, None, now),
        Waiting::Refused(1)
    );
    assert_eq!(decision(&mut seats, "nobody", 7, now), None);
}
