//! The numbers a state produces are a contract with every consumer's saved games.

use unfolded_tabletop::random::SeededRandom;

/// Wide enough that about half of the generator's values are rejected: the
/// eleven indexes below took fifteen steps.
const WIDE: usize = (1 << 63) + 1;
const RANGES: [usize; 11] = [1, 2, 6, 16, 33, 52, 1000, WIDE, WIDE, WIDE, WIDE];

/// Recorded from `arkham-unfolded` at 4e7a014, before its generator moved here:
/// from seed 89 it drew these indexes over `RANGES` and was left in this state.
const ARKHAM_INDEXES: [usize; 11] = [
    0,
    1,
    4,
    8,
    10,
    13,
    244,
    790_593_979_517_193_502,
    3_061_971_736_353_442_415,
    7_495_161_214_544_515_997,
    3_495_176_742_473_237_818,
];
const ARKHAM_STATE_AFTER_INDEXES: u64 = 4_990_025_626_462_012_820;

/// From that state it then shuffled a 34-card deck. Each entry is the place the
/// card held before the shuffle.
const ARKHAM_SHUFFLE: [usize; 34] = [
    9, 10, 21, 3, 2, 5, 26, 16, 11, 4, 14, 1, 19, 25, 13, 18, 29, 7, 6, 22, 20, 17, 8, 15, 24, 12,
    31, 33, 0, 23, 27, 32, 28, 30,
];
const ARKHAM_STATE_AFTER_SHUFFLE: u64 = 12_278_733_189_936_530_505;

#[test]
fn a_seed_draws_the_indexes_arkham_recorded_before_the_move() {
    let mut random = SeededRandom::new(89);
    let indexes = RANGES.map(|upper| random.index(upper));
    assert_eq!(
        (indexes, random.state()),
        (ARKHAM_INDEXES, ARKHAM_STATE_AFTER_INDEXES)
    );
}

#[test]
fn a_saved_state_resumes_into_the_shuffle_arkham_recorded_before_the_move() {
    let mut random = SeededRandom::new(ARKHAM_STATE_AFTER_INDEXES);
    let mut deck: Vec<usize> = (0..34).collect();
    random.shuffle(&mut deck);
    assert_eq!(
        (deck.as_slice(), random.state()),
        (ARKHAM_SHUFFLE.as_slice(), ARKHAM_STATE_AFTER_SHUFFLE)
    );
}

#[test]
fn every_index_is_below_its_range_and_a_small_range_reaches_all_of_them() {
    let mut random = SeededRandom::new(7);
    for upper in 1..=20 {
        let mut reached = vec![false; upper];
        for _ in 0..2_000 {
            reached[random.index(upper)] = true;
        }
        assert!(reached.iter().all(|reached| *reached), "range {upper}");
    }
}

#[test]
fn a_shuffle_keeps_every_item_and_fewer_than_two_items_draw_nothing() {
    let mut random = SeededRandom::new(7);
    let mut none: [char; 0] = [];
    let mut one = ['a'];
    random.shuffle(&mut none);
    random.shuffle(&mut one);
    assert_eq!((one, random.state()), (['a'], 7));

    let sorted: Vec<u32> = (0..100).collect();
    let mut items = sorted.clone();
    random.shuffle(&mut items);
    assert_ne!(items, sorted);
    items.sort_unstable();
    assert_eq!(items, sorted);
}

#[test]
#[should_panic(expected = "empty range")]
fn an_empty_range_is_refused() {
    SeededRandom::new(7).index(0);
}
