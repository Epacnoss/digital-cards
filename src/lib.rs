#![warn(clippy::pedantic, clippy::all, clippy::nursery)]
#![allow(clippy::missing_panics_doc)]

pub mod cheat;
pub mod game_type;
pub mod message_parser;
pub mod mpmc;

use std::net::{TcpListener, TcpStream};

pub use message_parser::*;

use cardpack::{Card, Rank, Suit};

pub const PORT: u16 = 6464;
pub const LOCAL_SERVER: bool = false;
pub const TPS_TIMER: u64 = 500;

#[must_use]
pub fn get_ip() -> String {
    if LOCAL_SERVER {
        format!("127.0.0.1:{}", PORT)
    } else {
        format!("139.162.229.144:{}", PORT)
    }
}

#[must_use]
pub fn parse_card(card: impl Into<String>) -> Option<Card> {
    use cardpack::{
        ACE, CLUBS, DIAMONDS, EIGHT, FIVE, FOUR, HEARTS, JACK, KING, NINE, QUEEN, SEVEN, SIX,
        SPADES, TEN, THREE, TWO,
    };

    let card_string = card.into();
    if card_string.is_empty() {
        return None;
    }

    let card_str = card_string.as_str();

    let rank: &'static str = match &card_str[0..1] {
        "2" => TWO,
        "3" => THREE,
        "4" => FOUR,
        "5" => FIVE,
        "6" => SIX,
        "7" => SEVEN,
        "8" => EIGHT,
        "9" => NINE,
        "T" => TEN,
        "J" => JACK,
        "Q" => QUEEN,
        "K" => KING,
        _ => ACE,
    };
    let suit: &'static str = match &card_str[1..2] {
        "S" => SPADES,
        "C" => CLUBS,
        "H" => HEARTS,
        _ => DIAMONDS,
    };

    Some(Card::new(Rank::new(rank), Suit::new(suit)))
}

#[must_use]
pub fn parse_pile(input: impl Into<String>) -> Vec<Card> {
    input.into().split(' ').filter_map(parse_card).collect()
}

#[cfg(test)]
pub mod tests {
    use crate::{parse_card, parse_pile};
    use cardpack::Pack;

    #[test]
    fn check_card_parser() {
        let card = Pack::french_deck().cards().shuffle().draw_first().unwrap();
        let parsed_card = parse_card(&format!("{}", card));

        assert_eq!(card, parsed_card.unwrap());
    }

    #[test]
    fn check_pile_parser() {
        let pile = Pack::french_deck().cards().shuffle();
        let parsed_pile = parse_pile(format!("{}", pile));

        assert_eq!(pile.cards(), &parsed_pile);
    }
}
