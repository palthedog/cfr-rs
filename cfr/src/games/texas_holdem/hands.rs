use iota::iota;
use log::debug;

use std::{
    cmp::Reverse,
    fmt,
    str::FromStr,
};

use super::*;

/*
 * Hand score is calculated as:
 *     <type> <rank> * 5
 * Type: 4 bits
 * Rank: 4 bits
 * IN total: 4 + 4 * 5 = 24 bits
 *
 * The important card (e.g. pair) is located at MSB.
 * Remaining <rank>s are sorted by rank.
 *
 * Fold             : 1
 * Empty            : 2
 * High card        : 3 rank * 5
 * One pair of x   : 4 xxxx xxxx rank * 3
 * Two pair of x, y: 5 xxxx xxxx yyyy yyyy rank (x > y)
 * Three of a x    : 6 xxxx xxxx xxxx rank * 2
 * Straight        : 7 rank * 5
 *   (For straight of 5-A, A should be located at LSB)
 * Flash           : 8 rank * 5
 * Full house      : 9 xxxx xxxx xxxx yyyy yyyy (x > y)
 * Four of a kind  : A xxxx xxxx xxxx xxxx rank
 * Straight flash  : B rank * 5
*/

const RANK_SIZE_BITS: u32 = 4;
const HAND_TYPE_SHIFT_BITS: u32 = RANK_SIZE_BITS * 5;

type HandType = u32;
const HAND_TYPE_MASK: u32 = !0 << HAND_TYPE_SHIFT_BITS;
const RANK_MASK: u32 = 0b1111;
iota! {
    const FOLD: HandType = iota << HAND_TYPE_SHIFT_BITS;
        , EMPTY
        , HIGH_CARD
        , ONE_PAIR
        , TWO_PAIR
        , THREE_OF_KIND
        , STRAIGHT
        , FLASH
        , FULL_HOUSE
        , FOUR_OF_KIND
        , STRAIGHT_FLASH
}

fn hand_type_name(t: HandType) -> String {
    (match t {
        FOLD => "Fold",
        EMPTY => "Empty",
        HIGH_CARD => "High card",
        ONE_PAIR => "One pair",
        TWO_PAIR => "Two pair",
        THREE_OF_KIND => "Three or kind",
        STRAIGHT => "Straight",
        FLASH => "Flash",
        FULL_HOUSE => "Full house",
        FOUR_OF_KIND => "Four of kind",
        STRAIGHT_FLASH => "Straight flash",
        _ => "Err hand",
    })
    .to_string()
}

#[derive(Copy, Clone, PartialOrd, PartialEq, Ord, Eq)]
pub struct HandScore {
    pub value: u32,
}

impl fmt::Display for HandScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.hand_name();

        let mut rs = String::new();
        for i in 0..5 {
            let shift = RANK_SIZE_BITS * (4 - i);
            let rank = ((self.value >> shift) & RANK_MASK) as u8;
            rs.push(rank_ch(rank));
        }
        write!(f, "{}-{}", name, rs)
    }
}

impl fmt::Debug for HandScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for HandScore {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(calc_hand_score(&parse_cards(s)))
    }
}

impl HandScore {
    pub fn fold() -> HandScore {
        HandScore {
            value: FOLD,
        }
    }
    pub fn empty() -> HandScore {
        HandScore {
            value: EMPTY,
        }
    }
    fn new(hand_type: HandType, cards: &[Card]) -> HandScore {
        let mut value = hand_type;
        let mut sht = RANK_SIZE_BITS * 5;
        for card in cards.iter().take(5) {
            sht -= RANK_SIZE_BITS;
            value |= u32::from(card.rank) << sht;
        }
        HandScore {
            value,
        }
    }

    fn hand_type(&self) -> HandType {
        self.value & HAND_TYPE_MASK
    }

    fn hand_name(&self) -> String {
        hand_type_name(self.hand_type())
    }

    fn with_hand_type(&self, hand_type: HandType) -> HandScore {
        let value = (self.value & !HAND_TYPE_MASK) | hand_type;
        HandScore {
            value,
        }
    }
}

pub fn calc_player_score(s: &HandState, player: &PlayerState) -> HandScore {
    let mut cs = player.hole_cards.to_vec();
    cs.append(&mut s.community_cards.to_vec());
    debug!("player: {:?}, cards: {:?}", player, cs);
    calc_hand_score(&cs)
}

fn make_round_by_rank(sorted_by_rank: &[Card]) -> Vec<Card> {
    if sorted_by_rank.is_empty() {
        return vec![];
    }
    if sorted_by_rank[0].rank != 14 || sorted_by_rank.last().unwrap().rank != 2 {
        return sorted_by_rank.to_vec();
    }
    let aces = sorted_by_rank.iter().filter(|c| c.rank == 14);
    sorted_by_rank.iter().chain(aces).cloned().collect()
}

fn sort_cards_by_rank(cards: &[Card]) -> Vec<Card> {
    let mut sorted_by_rank = cards.to_vec();
    sorted_by_rank.sort_by(|a, b| b.rank.cmp(&a.rank));
    sorted_by_rank
}

pub fn calc_score(hs: &[Card], cs: &[Card]) -> HandScore {
    let cards = hs.iter().chain(cs).cloned().collect::<Vec<_>>();
    calc_hand_score(&cards)
}

/*
 * Calculates the hand's score
*/
pub fn calc_hand_score(cards: &[Card]) -> HandScore {
    if cards.len() < 5 {
        return HandScore::empty();
    }
    let sorted_by_rank = sort_cards_by_rank(cards);
    let groups_by_suit = group_by_suit(&sorted_by_rank);
    let groups_by_rank = group_by_rank(&sorted_by_rank);
    let rounded = make_round_by_rank(&sorted_by_rank);

    if let Some(s) = is_straight_flash(&groups_by_suit) {
        return s;
    }
    if let Some(s) = is_four_of_kind(&groups_by_rank) {
        return s;
    }
    if let Some(s) = is_full_house(&groups_by_rank) {
        return s;
    }
    if let Some(s) = is_flash(&groups_by_suit) {
        return s;
    }
    if let Some(s) = is_straight(&rounded) {
        return s;
    }
    if let Some(s) = is_three_of_kind(&groups_by_rank) {
        return s;
    }
    if let Some(s) = is_two_pair(&groups_by_rank) {
        return s;
    }
    if let Some(s) = is_one_pair(&groups_by_rank) {
        return s;
    }
    if let Some(s) = is_high_card(&sorted_by_rank) {
        return s;
    }
    panic!("Failed to calculate score for {:?}.", cards);
}

pub fn is_four_of_kind(groups_by_rank: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_rank.is_empty() {
        return None;
    }
    if groups_by_rank[0].len() == 4 {
        let mut cs: Vec<Card> = flatten_groups(&groups_by_rank[0..1]);
        cs.push(flatten_and_sort_groups(&groups_by_rank[1..])[0]);
        Some(HandScore::new(FOUR_OF_KIND, &cs))
    } else {
        None
    }
}

pub fn is_full_house(groups_by_rank: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_rank.len() < 2 {
        return None;
    }
    if groups_by_rank[0].len() >= 3 && groups_by_rank[1].len() >= 2 {
        let three = groups_by_rank[0].iter().take(3);
        let two = groups_by_rank[1].iter().take(2);
        let cs: Vec<Card> = three.chain(two).cloned().collect();
        Some(HandScore::new(FULL_HOUSE, &cs))
    } else {
        None
    }
}

pub fn is_flash(groups_by_suit: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_suit[0].len() >= 5 {
        Some(HandScore::new(FLASH, &groups_by_suit[0]))
    } else {
        None
    }
}

pub fn is_straight_flash(groups_by_suit: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_suit.is_empty() || groups_by_suit[0].len() < 5 {
        return None;
    }

    let rounded = make_round_by_rank(&groups_by_suit[0]);
    is_straight(&rounded).map(|straight| straight.with_hand_type(STRAIGHT_FLASH))
}

pub fn is_straight(rounded: &[Card]) -> Option<HandScore> {
    let mut cur = 99;
    let mut next = 99;
    let mut connected = 0;
    let mut connected_cards = [Card::dummy(); 5];
    for &card in rounded {
        if card.rank == cur {
            // same rank.
            continue;
        }

        if card.rank != next {
            connected = 0;
        }
        connected_cards[connected] = card;
        connected += 1;
        if connected == 5 {
            return Some(HandScore::new(STRAIGHT, &connected_cards));
        }

        cur = card.rank;
        next = if card.rank == 2 {
            14
        } else {
            card.rank - 1
        };
    }
    None
}

pub fn is_three_of_kind(groups_by_rank: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_rank.is_empty() {
        return None;
    }
    if groups_by_rank[0].len() == 3 {
        let mut cs: Vec<Card> = groups_by_rank[0].clone();
        cs.extend(&flatten_and_sort_groups(&groups_by_rank[1..]));
        return Some(HandScore::new(THREE_OF_KIND, &cs));
    }
    None
}

pub fn is_two_pair(groups_by_rank: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_rank.len() < 2 {
        return None;
    }
    if groups_by_rank[0].len() == 2 && groups_by_rank[1].len() == 2 {
        let mut cs: Vec<Card> = flatten_groups(&groups_by_rank[0..2]);
        cs.push(flatten_and_sort_groups(&groups_by_rank[2..])[0]);
        return Some(HandScore::new(TWO_PAIR, &cs));
    }
    None
}

pub fn is_one_pair(groups_by_rank: &[Vec<Card>]) -> Option<HandScore> {
    if groups_by_rank.is_empty() {
        return None;
    }
    if groups_by_rank[0].len() == 2 {
        let mut cs: Vec<Card> = groups_by_rank[0].clone();
        cs.extend(&flatten_and_sort_groups(&groups_by_rank[1..]));
        return Some(HandScore::new(ONE_PAIR, &cs));
    }
    None
}

pub fn is_high_card(by_rank: &[Card]) -> Option<HandScore> {
    Some(HandScore::new(HIGH_CARD, by_rank))
}

fn flatten_groups(groups: &[Vec<Card>]) -> Vec<Card> {
    let mut ret = vec![];
    for group in groups {
        for card in group {
            ret.push(*card);
        }
    }
    ret
}

fn flatten_and_sort_groups(groups: &[Vec<Card>]) -> Vec<Card> {
    sort_cards_by_rank(&flatten_groups(groups))
}

/*
 * Makes a group by rank.
 * The groups are sorted by the size of the group.
*/
fn group_by_rank(by_rank: &[Card]) -> Vec<Vec<Card>> {
    if by_rank.is_empty() {
        return vec![];
    }

    let mut it = by_rank.iter();
    let card = *it.next().unwrap();
    let mut groups = vec![];
    let mut g = vec![card];
    for card in it {
        if g[0].rank == card.rank {
            g.push(*card);
        } else {
            groups.push(g);
            g = vec![*card];
        }
    }
    groups.push(g);
    groups.sort_by_key(|b| Reverse(b.len()));
    groups
}

/*
 * Makes a group by suit.
 * The groups are sorted by the size of the group.
 * Also, cards in each groups are sorted by rank.
*/
fn group_by_suit(by_rank: &[Card]) -> Vec<Vec<Card>> {
    if by_rank.is_empty() {
        return vec![];
    }
    let mut groups: Vec<Vec<Card>> = vec![vec![], vec![], vec![], vec![]];
    for card in by_rank {
        let index = card.suit as usize;
        groups[index].push(*card);
    }
    groups.sort_by_key(|b| Reverse(b.len()));
    groups
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_group_by_rank_empty() {
        let groups = group_by_rank(&[]);
        assert!(groups.is_empty());
    }

    #[test]
    fn test_group_by_rank() {
        let cards = &parse_cards("Th8hKdKs8dAhKh");
        let sorted_by_rank = sort_cards_by_rank(cards);
        let groups = group_by_rank(&sorted_by_rank);
        // 3 Kings
        assert_eq!(3, groups[0].len());
        assert_eq!(13, groups[0][0].rank);
        assert_eq!(13, groups[0][1].rank);
        assert_eq!(13, groups[0][2].rank);

        // 2 8s
        assert_eq!(2, groups[1].len());
        assert_eq!(8, groups[1][0].rank);
        assert_eq!(8, groups[1][1].rank);

        // 1 Ace
        assert_eq!(1, groups[2].len());
        assert_eq!(14, groups[2][0].rank);

        // 10
        assert_eq!(1, groups[3].len());
        assert_eq!(10, groups[3][0].rank);
    }

    #[test]
    fn test_group_by_rank_three_pairs() {
        let cards = &parse_cards("Td3s 8cThAh 8d 3h");
        let sorted_by_rank = sort_cards_by_rank(cards);
        let groups = group_by_rank(&sorted_by_rank);
        // T * 2
        assert_eq!(2, groups[0].len());
        assert_eq!(10, groups[0][0].rank);
        assert_eq!(10, groups[0][1].rank);

        // 8 * 2
        assert_eq!(2, groups[1].len());
        assert_eq!(8, groups[1][0].rank);
        assert_eq!(8, groups[1][1].rank);

        // 3 * 2
        assert_eq!(2, groups[2].len());
        assert_eq!(3, groups[2][0].rank);
        assert_eq!(3, groups[2][1].rank);

        // A
        assert_eq!(1, groups[3].len());
        assert_eq!(14, groups[3][0].rank);
    }

    #[test]
    fn test_group_by_suit() {
        let groups = group_by_suit(&parse_cards("As Kh Qh Js Th 8h 8d"));
        assert_eq!(4, groups.len());

        assert_eq!(4, groups[0].len()); // heart
        assert_eq!(2, groups[1].len()); // spade
        assert_eq!(1, groups[2].len()); // diamond
        assert_eq!(0, groups[3].len()); // club
    }

    #[test]
    fn test_rounded() {
        let r = make_round_by_rank(&parse_cards("Kh Jh Th 8h 7h 2h 2h"));
        assert_eq!(7, r.len());

        let r = make_round_by_rank(&parse_cards("Ah Kh Jh Th 8h 7h 2h"));
        assert_eq!(8, r.len());
        assert_eq!(14, r[0].rank);
        assert_eq!(14, r[7].rank);

        // We don't need to round the cards if the last one is not 2.
        let r = make_round_by_rank(&parse_cards("Ah Kh Jh Th 8h 7h 4h"));
        assert_eq!(7, r.len());

        // We don't need to round the cards if the first one is not A.
        let r = make_round_by_rank(&parse_cards("Kh Kd Jh Th 8h 7h 2h"));
        assert_eq!(7, r.len());
    }

    #[allow(dead_code)]
    fn check_hand_type(t: HandType, txt: &str) {
        let score = HandScore::from_str(txt).unwrap();
        let hand_type = score.hand_type();
        assert_eq!(
            t,
            hand_type,
            "\nExpected(left): {}\nActual(right): {} ({})",
            hand_type_name(t),
            hand_type_name(hand_type),
            txt
        );
    }

    #[allow(dead_code)]
    fn hand_eq(a: &str, b: &str) {
        let score_a = HandScore::from_str(a).unwrap();
        let score_b = HandScore::from_str(b).unwrap();
        assert_eq!(score_a, score_b, "\nExpected {} == {}", score_a, score_b);
    }

    #[allow(dead_code)]
    fn hand_gt(a: &str, b: &str) {
        let score_a = HandScore::from_str(a).unwrap();
        let score_b = HandScore::from_str(b).unwrap();
        assert!(
            score_a > score_b,
            "\nExpected {}({}) is greater than {}({})",
            score_a,
            a,
            score_b,
            b
        );
    }

    #[test]
    fn test_hand_type() {
        assert_ne!(ONE_PAIR, TWO_PAIR);
        assert_ne!(TWO_PAIR, THREE_OF_KIND);
    }

    #[test]
    fn test_straight_flash() {
        check_hand_type(STRAIGHT_FLASH, "Ah Kh Qh Jh Th");
        check_hand_type(STRAIGHT_FLASH, "Ah Kh Qh Jh Th 8s 7s");
        check_hand_type(STRAIGHT_FLASH, "Ah Kh Qh Jh Th 8h 7h");

        // Duplicated Q.
        check_hand_type(STRAIGHT_FLASH, "Kh Qh Qs Jh Th 9h 8h");
        check_hand_type(STRAIGHT_FLASH, "Ks Qh Qs Jh Th 9h 8h");

        // Rounded
        check_hand_type(STRAIGHT_FLASH, "5h 4h 3h 2h Ah Kh Qh");
    }

    #[test]
    fn test_four_of_kind() {
        check_hand_type(FOUR_OF_KIND, "Ah Ad Ac As Kc Kd 8h");
        check_hand_type(FOUR_OF_KIND, "Kh Kd Kc Ks Qc Td 8h");

        // Four of kind. Not Full-house.
        check_hand_type(FOUR_OF_KIND, "Kh Kd Kc Ks Qc Qd Qh");

        // 4 cards + 2 cards + A
        hand_eq("Ah Kh Kd Kc Ks Qc Qd", "Ah Kh Kd Kc Ks");
        // 4 cards + 2 cards + 3
        hand_eq("3h Kh Kd Kc Ks Qc Qd", "Qh Kh Kd Kc Ks");
    }

    #[test]
    fn test_four_of_kind_ignored_pocket_pair() {
        hand_eq("7d 7s 9s 3c 3d 3h 3s", "8d 8s 9s 3c 3d 3h 3s");
    }

    #[test]
    fn test_full_house() {
        check_hand_type(FULL_HOUSE, "Ah As Ac Ks Kc Kd 8h");
        check_hand_type(FULL_HOUSE, "Ah As Ac Ks Kc 8h 7s");
    }

    #[test]
    fn test_full_house_score() {
        let unused_j = "Ah As Ac 8s 8c Jh 5s";
        let unused_6 = "Ah As Ac 8s 8c 6h 5s";
        hand_eq(unused_j, unused_6);

        let high = "9h 9s Ah As Ac 8s 8c";
        let low_ = "9h 5s Ah As Ac 8s 8c";
        hand_gt(high, low_);
    }

    #[test]
    fn test_flash() {
        let nut = "Ah Th 9h 8h 7h Qs Tc";
        let weak = "5h 2h 9h 8h 7h Qs Tc";
        check_hand_type(FLASH, nut);
        check_hand_type(FLASH, weak);
        hand_gt(nut, weak);

        let more_than_5 = "5h 2h 9h 8h 7h Qh Th";
        check_hand_type(FLASH, more_than_5);

        let with_a = "Ah 2h 9h 8h 7h Qh Th";
        let with_t = "5h 2h 9h 8h 7h Qh Th";
        hand_gt(with_a, with_t);
    }

    #[test]
    fn test_straight() {
        check_hand_type(STRAIGHT, "Ah Kh Qd Jh Th 8d 7c");
        check_hand_type(STRAIGHT, "Ad Ac Kh Qd Jh Th 8d");
        check_hand_type(STRAIGHT, "Ah Kh Qd Qd Jh Th 8d");
        check_hand_type(STRAIGHT, "5h 5s 5d 4h 3h 2d Ah");

        // Shuffled
        check_hand_type(STRAIGHT, "Qd 8d Jh Th Kh 7h 9d");
    }

    #[test]
    fn test_straight_score() {
        // A ~ T
        let nut = "Ah Kh Qd Jh Th 8d 7s";
        let middle = "Qd Jh Th 9h 8d 7h 3s";
        // 5 ~ 1
        let weak = "5h 5s 5d 4h 3h 2d Ah";
        check_hand_type(STRAIGHT, nut);
        check_hand_type(STRAIGHT, middle);
        check_hand_type(STRAIGHT, weak);
        hand_gt(nut, middle);
        hand_gt(nut, weak);
        hand_gt(middle, weak);

        let high = "7s 5d Ah 3d 6h 8s 4c";
        let low = "2s 5h Ah 3d 6h 8s 4c";
        check_hand_type(STRAIGHT, high);
        check_hand_type(STRAIGHT, low);
        hand_gt(high, low);
    }

    #[test]
    fn test_three_of_kind() {
        check_hand_type(THREE_OF_KIND, "Ah Ad Kh Qd Th As 7c");
        check_hand_type(THREE_OF_KIND, "Ah Kd Qc 9d Th As Ad");
    }

    #[test]
    fn test_two_pair() {
        check_hand_type(TWO_PAIR, "Ah Ad Kh Kd Th 8h 7c");
        check_hand_type(TWO_PAIR, "Ah Th 8h Th Ad");
    }

    #[test]
    fn test_two_pair_kicker() {
        let ace_kicker = "Ah 3d  Kh Kd Th 8h 7c";
        let j_kicker = "Jh 4d  Kh Kd Ts 8h 7c";
        hand_gt(ace_kicker, j_kicker);

        // Kickers are not chosen from their hole cards.
        hand_eq("5h 3d  2s 2d Ts Td 7c", "4h 3d  2h 2d Th Td 7h");
        hand_eq("5h 3d  Ks Kd Tc Td 7h", "4h 3d  Kh Kd Tc Td 7s");

        //// 3 Pairs
        hand_eq(
            "Ah TdTh 8d8c 5h5s", // Kicker should be A
            "Ah TdTh 8d8c",
        );
        hand_eq(
            "3h TdTh 8d8c 5h5s", // Kicker should be 5
            "5h TdTh 8d8c",
        );
    }

    #[test]
    fn test_one_pair() {
        check_hand_type(ONE_PAIR, "Ah Ad Kh Js Ts 8h 7h");
        check_hand_type(ONE_PAIR, "Ah Th 8h 7h Ad");
    }

    #[test]
    fn test_high_card() {
        check_hand_type(HIGH_CARD, "Ah Kc Js Th 8c 7s 5h");
        check_hand_type(HIGH_CARD, "5h 8h 7d Kc As");

        // Wrong straight.
        check_hand_type(HIGH_CARD, "9h 8s 4h 3h 2d Ah Kd");
    }

    #[test]
    fn test_calc_hand_score() {
        let one_pair = "Kh Kd Qh Jh Th";
        let high_card_a = "Ah Kd Qh 5h 9h";
        let high_card_q = "Qh 5d 9h 7h 4h";
        hand_gt(one_pair, high_card_a);
        hand_gt(high_card_a, high_card_q);
    }
}
