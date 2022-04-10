use cardpack::{Pack, Pile};
use digital_cards::{parse_pile, test_config, MessageToClient, MessageToServer};
use networking::{error::NetworkError, syncronous::{SyncHost, SyncDataStream}, ConnectionRequest};
use parking_lot::Mutex;
use std::{sync::Arc};
use crossbeam::channel::unbounded;
use std::convert::TryInto;

fn main() {
    pretty_logger::init_to_defaults().unwrap();

    let (peer, config) = test_config();
    let host = SyncHost::from_host_data(&config).unwrap();

    let cards: Arc<Mutex<Pile>> = Arc::new(Mutex::new({
        let mut end = vec![];
        let base_pack = Pack::french_deck();
        let cards = base_pack.cards().cards();
        for _ in 0..5 {
            cards.clone().into_iter().for_each(|card| end.push(card));
        }
        Pile::from_vector(end).shuffle()
    }));

    let pile: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));
    
    let (streams_tx, streams_rx) = unbounded();
    
    std::thread::spawn(move || {
        let mut streams_buffer = vec![];
        for netstream in host {
            let stream = netstream.unwrap().verify(&peer.clone()).unwrap();
            
            if streams_buffer.is_empty() {
                streams_buffer.push(stream);
            } else {
                streams_tx.send((
                    streams_buffer.remove(0),
                    stream
                    )).unwrap();
            }
        }
    });

    for (mut processing_stream, mut recv_stream) in streams_rx.iter() {
        let cards = cards.clone();
        let pile = pile.clone();
        std::thread::spawn(move || {
            log::info!("new connection from {:?}", processing_stream.addr());
        
            let mut buffer;
            loop {
            
                buffer = vec![];
                log::trace!("Waiting for input");
                if let Err(network_error) = processing_stream.recv(&mut buffer) {
                    match network_error {
                        NetworkError::IOError(io_error) => {
                            log::error!("IOError: {}", io_error);
                        }
                        _ => log::error!("Network Error: {}", network_error),
                    }
                    return;
                }
            
                log::info!("Client sent data: {:?}", &buffer);
                let msg: MessageToServer = buffer.remove(0).try_into().unwrap();
                log::info!("Client sent message: {:?}", &msg);
            
                match msg {
                    MessageToServer::AddingToPile => {
                        let from_client = parse_pile(String::from_utf8(buffer).unwrap());
                        let mut pile = pile.lock();
                        from_client.into_iter().for_each(|card| pile.push(card));
    
                        let mut vec = vec![MessageToClient::CurrentPileFollows as u8; 1];
                        format!("{}", pile)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));
    
                        recv_stream.send(&vec).unwrap();
                    }
                    MessageToServer::Draw1 | MessageToServer::Draw2 | MessageToServer::Draw3 => {
                        let cards_to_draw = msg as u8 - 199;
                        log::info!("Drawing {} cards", cards_to_draw);
                        let mut cards = cards.lock();
                        let cards_drew = cards.draw(cards_to_draw as usize).unwrap_or_else(|| {
                            log::info!("Deck to draw from now empty!");
                            Pile::default()
                        });
                    
                        let mut vec = vec![MessageToClient::SendingCardsToHand as u8; 1];
                        format!("{}", cards_drew)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));
                        
                        recv_stream.send(&vec).unwrap();
                    
                        log::info!("Sent new cards to client: {:?}", &vec);
                    }
                    MessageToServer::SendCurrentPilePlease => {
                        let pile = pile.lock();
                    
                        let mut vec = vec![MessageToClient::CurrentPileFollows as u8; 1];
                        format!("{}", pile)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));
                        recv_stream.send(&vec).unwrap();
                    
                        log::info!("Sent pile with data {:?}", &vec);
                    }
                    _ => {}
                }
            }
        });
    }
}
