#![warn(clippy::pedantic, clippy::all, clippy::nursery)]

pub mod message_parser;
pub mod mpmc;

pub use message_parser::*;

use cardpack::{Card, Rank, Suit};
use networking::{
    encryption::PubKeyComp, get_private_key, ArtificeConfig, ArtificeHostData, ArtificePeer,
    Layer3Addr, Layer3SocketAddr,
};

pub const PORT: u16 = 6464;

#[must_use]
///Uses WB3 in Wales as host
pub fn test_config(is_server: bool) -> ArtificeConfig {
    let host_addr = Layer3SocketAddr::new(
        if !is_server {
            Layer3Addr::newv4(81, 151, 40, 2)
        } else {
            Layer3Addr::newv4(127, 0, 0, 1)
        },
        PORT,
    );

    let private_key = get_private_key();
    let host_hash = "f7Cgkll1EegEa5UyuUEADpYAXRXwrhbSB0FLLiYxHpBotzNrw9";

    let host_data = ArtificeHostData::new(&private_key, host_hash);
    ArtificeConfig::new(host_addr, host_data, false)
}

#[must_use]
///For use by clients
/// The client address is the address of the machine calling stuff.
pub fn test_config_peer(client: Layer3Addr, is_server: bool) -> (ArtificePeer, ArtificeConfig) {
    let client = Layer3SocketAddr::new(client, 6464);

    let private_key = get_private_key();
    let pubkey = PubKeyComp::from(&private_key);
    // poorly named, global is unique to each host, and peer hash is a pre-shared key
    let peer_hash = "7VKkjONo1txtTAiR1vQWUTsGxh8jwQJips1ClMv9zv1CsOo3ZX";
    let remote_hash = "73C0YnEJRpTd56wPwR8zHa3egpW8iM1ShCRAtutkcssenNkJ0T";

    let peer = ArtificePeer::new(remote_hash, peer_hash, client, Some(pubkey));
    (peer, test_config(is_server))
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
