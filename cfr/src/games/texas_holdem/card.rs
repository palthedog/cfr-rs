use std::{
    char,
    fmt,
    str,
};

pub type Rank = u8;

pub fn rank_ch(r: Rank) -> char {
    match r {
        10 => 'T',
        11 => 'J',
        12 => 'Q',
        13 => 'K',
        14 => 'A',
        x => (b'0' + x).into(),
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Suit {
    Spade,
    Club,
    Heart,
    Diamond,
}

pub const SUITS: [Suit; 4] = [Suit::Spade, Suit::Club, Suit::Heart, Suit::Diamond];

pub fn suit_ch(s: Suit) -> char {
    match s {
        Suit::Spade => 's',
        Suit::Club => 'c',
        Suit::Heart => 'h',
        Suit::Diamond => 'd',
    }
}

pub fn cards_to_str(cards: &[Card]) -> String {
    let mut s = "".to_string();
    for card in cards.iter() {
        s.push_str(&card.to_string());
    }
    s
}

pub fn parse_cards(s: &str) -> Vec<Card> {
    let s = s.replace(' ', "");
    let s = s.replace(',', "");

    assert_eq!(s.len() % 2, 0, "Given string is {}", s);

    let mut cards: Vec<Card> = Vec::default();
    let mut i = 0;
    let size = s.len();
    while i < size {
        let cs: &str = &s[i..i + 2];
        cards.push(cs.parse().unwrap());
        i += 2
    }
    cards
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn dummy() -> Card {
        Card {
            rank: 0,
            suit: Suit::Heart,
        }
    }

    pub fn str(&self) -> String {
        let mut s = String::with_capacity(2);
        s.push(rank_ch(self.rank));
        s.push(suit_ch(self.suit));
        s
    }
}

impl str::FromStr for Card {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        assert_eq!(2, s.len());

        let mut cs = s.chars();
        let rank = cs.next().unwrap();
        let suit = cs.next().unwrap();
        let rank = match rank {
            'T' => 10,
            'J' => 11,
            'Q' => 12,
            'K' => 13,
            'A' => 14,
            x => {
                if !char::is_digit(x, 10) {
                    return Err(format!("Bad rank: {}", s));
                }
                x as u8 - b'0'
            }
        };
        let suit = match suit {
            's' => Suit::Spade,
            'c' => Suit::Club,
            'h' => Suit::Heart,
            'd' => Suit::Diamond,
            _ => return Err(format!("Bad suit: {}", s)),
        };
        Ok(Card {
            rank,
            suit,
        })
    }
}

impl fmt::Display for Card {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&self.str())
    }
}

impl fmt::Debug for Card {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(&self.str())
    }
}