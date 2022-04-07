#![warn(clippy::pedantic, clippy::all, clippy::nursery)]

use networking::{ArtificePeer, ArtificeConfig, Layer3SocketAddr, Layer3Addr, get_private_key, ArtificeHostData, encryption::PubKeyComp};
use cardpack::{Pile, Card, Rank, Suit};
use derive_try_from_primitive::TryFromPrimitive;

pub fn test_config() -> (ArtificePeer, ArtificeConfig) {
	let peer_addr: Layer3SocketAddr = Layer3SocketAddr::new(Layer3Addr::newv4(127, 0, 0, 1), 6464);
	let host_addr: Layer3SocketAddr = Layer3SocketAddr::new(Layer3Addr::newv4(127, 0, 0, 1), 6464);
	let private_key = get_private_key();
	let pubkey = PubKeyComp::from(&private_key);
	// poorly named, global is unique to each host, and peer hash is a pre-shared key
	let host_hash = "f7Cgkll1EegEa5UyuUEADpYAXRXwrhbSB0FLLiYxHpBotzNrw9";
	let peer_hash = "7VKkjONo1txtTAiR1vQWUTsGxh8jwQJips1ClMv9zv1CsOo3ZX";
	let remote_hash = "73C0YnEJRpTd56wPwR8zHa3egpW8iM1ShCRAtutkcssenNkJ0T";
	let peer = ArtificePeer::new(remote_hash, peer_hash, peer_addr, Some(pubkey));
	let host_data = ArtificeHostData::new(&private_key, host_hash);
	let config = ArtificeConfig::new(host_addr, host_data, false);
	(peer, config)
}

pub fn parse_pile (input: String) -> Vec<Card> {
	let mut pile = Vec::default();
	if input.is_empty() {
		return pile;
	}
	
	println!("Parsing: {}", input);
	
	for card in input.split(" ") {
		println!("Card: {}", card);
		
		use cardpack::{ACE, KING, QUEEN, JACK, TWO, THREE, FOUR, FIVE, SIX, SEVEN, EIGHT, NINE, TEN, SPADES, CLUBS, HEARTS, DIAMONDS};
		
		let rank: &'static str = match &card[0..1] {
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
			_ => ACE
		};
		let suit: &'static str = match &card[1..2] {
			"S" => SPADES,
			"C" => CLUBS,
			"H" => HEARTS,
			_ => DIAMONDS
		};
		
		
		pile.push(Card::new(Rank::new(rank), Suit::new(suit)));
	}
	
	pile
}


#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive)]
pub enum Message {
	Disconnect = 0,
	Connect = 1,
	SendingPile = 2,
}