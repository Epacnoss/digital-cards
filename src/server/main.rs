use cardpack::{Pack, Pile};
use digital_cards::{parse_pile, test_config, Message};
use networking::{syncronous::SyncHost, ConnectionRequest};
use std::convert::TryInto;
use std::sync::{Arc, RwLock};
use networking::error::NetworkError;

fn main() {
    let (peer, config) = test_config();
    let host = SyncHost::from_host_data(&config).unwrap();
    let cards = Arc::new(RwLock::new(Pack::french_deck().cards().shuffle()));
    let pile = Arc::new(RwLock::new(Pile::default()));

    for netstream in host {
        let peer = peer.clone();
        let cards = cards.clone();
        let pile = pile.clone();
        std::thread::spawn(move || {
            println!("new connection from {:?}", peer.addr());
            let mut stream = netstream.unwrap().verify(&peer).unwrap();

            {
                let mut cards = cards.write().unwrap_or_else(|err| {
                    eprintln!("Pile poisoned: {}", err);
                    stream
                        .send(&[Message::Disconnect as u8])
                        .unwrap_or_else(|err| {
                            eprintln!("Error sending disconnect message to client: {}", err);
                            1
                        });
                    std::process::exit(1);
                });

                let cards_drew = cards.draw(3).unwrap_or_default();

                let mut vec = vec![Message::SendingToHand as u8; 1];
                format!("{}", cards_drew)
                    .as_bytes()
                    .iter()
                    .for_each(|byte| vec.push(*byte));
                stream.send(&vec).unwrap_or_else(|err| {
                    eprintln!("Error sending pile to client: {}", err);
                    1
                });
                println!("Sent hand to client");
            }
	        
	        let mut buffer;
            loop {
	            
	            println!("Bout to get client data!");
                buffer = vec![];
                if let Err(network_error) = stream.recv(&mut buffer) {
                    match network_error {
                        NetworkError::IOError(io_error) => {
                            eprintln!("IOError: {}", io_error);
                            return;
                        },
                        _ => {}
                    }
                }
                
                
                println!("Received data from client: {:?}", &buffer);
                let msg: Message = buffer.remove(0).try_into().unwrap();
                println!("Client sent: {:?}", &msg);
	            
	            
                if msg == Message::SendingToDealerPile {
                    let from_client = parse_pile(String::from_utf8(buffer).unwrap());
                    let mut pile = pile.write().unwrap();
                    from_client.into_iter().for_each(|card| pile.push(card));

                    let mut buffer = vec![Message::CurrentPileIs as u8; 1];
                    format!("{}", pile)
                        .as_bytes()
                        .iter()
                        .for_each(|byte| buffer.push(*byte));
                    println!("Bout to send current pile to client: {:?}", &buffer);
                    stream.send(&buffer).unwrap();
                    println!("Sent current pile to client");
                }
            }
        });
    }

    println!("End of server")
}
