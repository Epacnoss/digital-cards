use networking::syncronous::{SyncHost, SyncStream};
// use std::{thread, time::Duration};
use cardpack::Pile;
use digital_cards::{parse_pile, test_config, MessageToClient, MessageToServer};
use networking::error::NetworkError;
use parking_lot::Mutex;
use std::{
    convert::TryInto,
    sync::{mpsc::channel, Arc},
};

fn main() {
    let (to_process_tx, to_process_rx) = channel();

    let (peer, config) = test_config();
    let host = SyncHost::client_only(&config).unwrap();
    
    let stream: Arc<Mutex<SyncStream>> = Arc::new(Mutex::new(host.connect(peer).unwrap()));
    let hand: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));

    let (processing_stream, processing_hand) = (stream.clone(), hand.clone());
    //processing thread
    std::thread::spawn(move || {
        log::trace!("CHILD: Processing thread started!");
        let hand = processing_hand;

        loop {
            for (msg, buffer) in to_process_rx.try_iter() {
                match msg {
                    MessageToClient::ServerEnd => {
                        log::warn!("CHILD: Server disconnected!");
                        break;
                    }
                    MessageToClient::SendingCardsToHand => {
                        log::log!("CHILD: Receiving cards: {:?}", &buffer);
                        let string = String::from_utf8(buffer).unwrap();
                        
                        let mut hand = hand.lock();
                        for card in parse_pile(string) {
                            hand.push(card);
                        }
                        log::log!("CHILD: Hand is now {}", hand);
                    }
                    MessageToClient::CurrentPileFollows => {
                        let string = String::from_utf8(buffer).unwrap();
                        let pile = Pile::from_vector(parse_pile(string));
                        let hand = processing_hand.lock();
                        log::info!("CHILD: Current pile from dealer is {}", pile);
                        log::info!("CHILD: Current hand is {}", hand);
                    }
                    _ => {}
                }

                let mut stream = processing_stream.lock();

                {
                    let mut hand = hand.lock();
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
                    stream
                        .send(&[MessageToServer::SendCurrentPilePlease as u8; 1])
                        .unwrap();
                }
            }
        }
    });

    {
        log::info!("MAIN: connected to server");

        {
            let mut stream = stream.lock();
            stream.send(&[MessageToServer::Draw3 as u8; 1]).unwrap();
        }

        let mut buffer;

        loop {
            log::trace!("MAIN: Start of recv loop");
            buffer = vec![];

            let mut stream = stream.lock();

            if let Err(network_error) = stream.recv(&mut buffer) {
                match network_error {
                    NetworkError::IOError(io_error) => {
                        log::error!("IOError: {}", io_error);
                        std::process::exit(1);
                    }
                    _ => {
                        log::error!("Other Error: {}", network_error);
                        std::process::exit(1);
                    }
                }
            }
            log::trace!("MAIN: Done waiting for stream!");

            let msg: MessageToClient = buffer.remove(0).try_into().unwrap();
            log::log!(
                "MAIN: Client received message, and sent to channel: {:?}",
                &msg
            );
            to_process_tx.send((msg, buffer)).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(100));
            log::trace!("MAIN: End of recv loop")
        }
    }
}