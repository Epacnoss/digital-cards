use networking::{syncronous::SyncHost, ConnectionRequest};
use digital_cards::{test_config, Message};
use std::sync::{Arc, RwLock};
use cardpack::{Pack, Pile, Card};

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
				stream.send(&[Message::Disconnect as u8]);
				std::process::exit(1);
			});
			
			let cards_drew = cards.draw(3).unwrap_or_default();
			
			let mut vec = vec![Message::SendingPile as u8; 1];
			format!("{}", cards_drew).clone().as_bytes().into_iter().for_each(|byte| vec.push(*byte));
			stream.send(&vec);
			
			let mut buff: Vec<u8> = Vec::new();
			stream.recv(&mut buff).unwrap();
			println!("Sent: {:?}, Received: {:?}", &vec, buff);
		});
	}
}