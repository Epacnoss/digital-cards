use networking::{syncronous::SyncHost, ConnectionRequest};
use digital_cards::test_config;

fn main () {
	let (peer, config) = test_config();
	let host = SyncHost::from_host_data(&config).unwrap();
	
	for netstream in host {
		let peer = peer.clone();
		std::thread::spawn(move || {
			
			println!("new connection from {:?}", peer.addr());
			let mut stream = netstream.unwrap().verify(&peer).unwrap();
			
			stream.send(b"hello world").unwrap();
			
			let mut buffer = Vec::new();
			stream.recv(&mut buffer).unwrap();
			println!("Got {}", String::from_utf8(buffer).unwrap());
			
		});
	}
}