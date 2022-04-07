use networking::syncronous::SyncHost;
// use std::{thread, time::Duration};
use cardpack::Pile;
use digital_cards::{parse_pile, test_config, Message};
use std::convert::TryInto;
use networking::error::NetworkError;

fn main() {
    let mut hand = Pile::default();
    let (peer, config) = test_config();
    let host = SyncHost::client_only(&config).unwrap();
    let mut stream = host.connect(peer).unwrap();
    println!("connected to server");

    let mut buffer;

    loop {
        buffer = vec![];
        stream.recv(&mut buffer).unwrap();
        let msg: Message = buffer.remove(0).try_into().unwrap();
        
        match msg {
            Message::Disconnect => {
                eprintln!("Server disconnected!");
                std::process::exit(0);
            }
            Message::SendingToHand => {
                let string = String::from_utf8(buffer).unwrap();

                for card in parse_pile(string) {
                    hand.push(card);
                }
                println!("Hand is {}", hand);
                let mut vec: Vec<u8> = vec![Message::SendingToDealerPile as u8; 1];
                format!("{}", hand)
                    .as_bytes()
                    .iter()
                    .for_each(|el| vec.push(*el));
                stream.send(&vec).unwrap_or_else(|err| {
                    eprintln!("Error sending data to server: {}", err);
                    1
                });
                println!("Sent hand to servr");
            }
            Message::CurrentPileIs => {
                let string = String::from_utf8(buffer).unwrap();
                println!(
                    "Current pile from dealer is {}",
                    Pile::from_vector(parse_pile(string))
                );
            }
            _ => {} // _ => std::thread::sleep(std::time::Duration::from_millis(50))
        }
    }
}
