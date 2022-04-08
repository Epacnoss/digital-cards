use networking::syncronous::{SyncHost, SyncStream};
// use std::{thread, time::Duration};
use cardpack::Pile;
use digital_cards::{parse_pile, test_config, MessageToClient, MessageToServer};
use std::convert::TryInto;
use networking::error::NetworkError;
use std::sync::mpsc::channel;
use std::sync::Arc;
use parking_lot::Mutex;

fn main() {
    let (to_process_tx, to_process_rx) = channel();
    
    let (peer, config) = test_config();
    let host = SyncHost::client_only(&config).unwrap();
    let stream: Arc<Mutex<SyncStream>> = Arc::new(Mutex::new(host.connect(peer).unwrap()));
    
    let processing_stream = stream.clone();
    //processing thread
    std::thread::spawn(move || {
        let mut hand = Pile::default();
        println!("CHILD: Processing thread started!");
    
        loop {
            for (msg, buffer) in to_process_rx.try_iter() {

                match msg {
                    MessageToClient::ServerEnd => {
                        println!("CHILD: Server disconnected!");
                        break;
                    },
                    MessageToClient::SendingCardsToHand => {
                        println!("CHILD: Receiving cards: {:?}", &buffer);
                        let string = String::from_utf8(buffer).unwrap();
            
                        for card in parse_pile(string) {
                            hand.push(card);
                        }
                        println!("CHILD: Hand is now {}", hand);
                    },
                    MessageToClient::CurrentPileFollows => {
                        let string = String::from_utf8(buffer).unwrap();
                        let pile = Pile::from_vector(parse_pile(string));
                        println!(
                            "CHILD: Current pile from dealer is {}",
                            pile
                        );
                        println!("CHILD: Current hand is {}", hand);
                    },
                    _ => {}
                }
    
                let mut stream = processing_stream.lock();
                
    
                {
                    let adding_to_pile = hand.draw(2);
                    if let Some(from_hand) = adding_to_pile {
                        let mut vec = vec![MessageToServer::AddingToPile as u8; 1];
                        format!("{}", from_hand)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));
    
                        // std::thread::sleep(std::time::Duration::from_millis(100));
                        stream.send(&vec).unwrap();
                    }
    
                    // std::thread::sleep(std::time::Duration::from_millis(100));
                    stream.send(&[MessageToServer::Draw3 as u8; 1]).unwrap();
                    std::thread::sleep(std::time::Duration::from_millis(25));
                    stream.send(&[MessageToServer::SendCurrentPilePlease as u8; 1]).unwrap();
                }
            }
        }
    });
    
    {
        println!("MAIN: connected to server");
        
        {
            let mut stream = stream.lock();
            stream.send(&[MessageToServer::Draw3 as u8; 1]).unwrap();
        }
    
    
        let mut buffer;
    
        loop {
            println!("MAIN: Start of recv loop");
            buffer = vec![];
    
            let mut stream = stream.lock();
    
            if let Err(network_error) = stream.recv(&mut buffer) {
                match network_error {
                    NetworkError::IOError(io_error) => {
                        eprintln!("IOError: {}", io_error);
                        std::process::exit(1);
                    },
                    _ => {
                        eprintln!("Other Error: {}", network_error);
                        std::process::exit(1);
                    }
                }
            }
            println!("MAIN: Done waiting for stream!");
    
            // println!("Server sent data: {:?}", &buffer);
            let msg: MessageToClient = buffer.remove(0).try_into().unwrap();
            println!("MAIN: Client received message, and sent to channel: {:?}", &msg);
            to_process_tx.send((msg, buffer)).unwrap();
    
            std::thread::sleep(std::time::Duration::from_millis(100));
            println!("MAIN: End of recv loop")
        }
        
    
    }
}
