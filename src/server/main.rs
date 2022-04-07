use networking::{syncronous::SyncHost, ConnectionRequest};
use digital_cards::{test_config, Message};
use std::sync::{Arc, RwLock};
use cardpack::Pack;

fn main () {
	let (peer, config) = test_config();
	let host = SyncHost::from_host_data(&config).unwrap();
	let cards = Arc::new(RwLock::new(Pack::french_deck().cards().shuffle()));
	
	for netstream in host {
		let peer = peer.clone();
		let cards = cards.clone();
		std::thread::spawn(move || {
			
			println!("new connection from {:?}", peer.addr());
			let mut stream = netstream.unwrap().verify(&peer).unwrap();
			
			let mut cards = cards.write().unwrap_or_else(|err| {
				eprintln!("Pile poisoned: {}", err);
				stream.send(&[Message::Disconnect as u8]).unwrap_or_else(|err| {
					eprintln!("Error sending disconnect message to client: {}", err);
					1
				});
				std::process::exit(1);
			});
			
			let cards_drew = cards.draw(3).unwrap_or_default();
			
			let mut vec = vec![Message::SendingPile as u8; 1];
			format!("{}", cards_drew).as_bytes().iter().for_each(|byte| vec.push(*byte));
			stream.send(&vec).unwrap_or_else(|err| {
				eprintln!("Error sending pile to client: {}", err);
				1
			});
			
			let mut buff: Vec<u8> = Vec::new();
			stream.recv(&mut buff).unwrap();
			println!("Sent: {:?}, Received: {:?}", &vec, buff);
		});
	}
}