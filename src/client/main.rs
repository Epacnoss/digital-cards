use networking::{syncronous::SyncHost};
// use std::{thread, time::Duration};
use digital_cards::{test_config, parse_pile, Message};
use cardpack::Pile;
use std::convert::TryInto;

fn main () {
	let mut hand = Pile::default();
	let (peer, config) = test_config();
	let host = SyncHost::client_only(&config).unwrap();
	let mut stream = host.connect(peer).unwrap();
	println!("connected");
	
	let mut buffer = Vec::new();
	stream.recv(&mut buffer).unwrap();
	let msg: Message = buffer.remove(0).try_into().unwrap();
	match msg {
		Message::Disconnect => {
			eprintln!("Server disconnected!");
			std::process::exit(0);
		}
		Message::SendingPile => {
			let string = String::from_utf8(buffer).unwrap();
			
			println!("got message: {} from server", &string);
			for card in parse_pile(string) {
				hand.push(card);
			}
			println!("Hand is {}", hand);
			let mut vec: Vec<u8> = vec![Message::SendingPile as u8; 1];
			format!("{}", hand).as_bytes().iter().for_each(|el| vec.push(*el));
			stream.send(&vec).unwrap_or_else(|err| {
				eprintln!("Error sending data to server: {}", err);
				1
			});
		}
	}
}