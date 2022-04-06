use networking::{syncronous::SyncHost};
// use std::{thread, time::Duration};
use digital_cards::test_config;
use std::time::Duration;

fn main () {
	let (peer, config) = test_config();
	let host = SyncHost::client_only(&config).unwrap();
	let mut stream = host.connect(peer).unwrap();
	println!("connected");
	
	let mut buffer = Vec::new();
	stream.recv(&mut buffer).unwrap();
	let string = String::from_utf8(buffer).unwrap();
	
	println!("got message: {} from server", string);
	
	std::thread::sleep(Duration::from_secs_f64(5.0));
	
	stream.send(b"Hey Yourself!").unwrap();
}