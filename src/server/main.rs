use cardpack::{Pack, Pile};
use digital_cards::{parse_pile, test_config, MessageToClient, MessageToServer};
use networking::{error::NetworkError, syncronous::SyncHost, ConnectionRequest};
use parking_lot::Mutex;
use std::{convert::TryInto, sync::Arc};

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

    for netstream in host {
        let peer = peer.clone();
        let cards = cards.clone();
        let pile = pile.clone();
        std::thread::spawn(move || {
            log::info!("new connection from {:?}", peer.addr());
            let mut stream = netstream.unwrap().verify(&peer).unwrap();

            let mut buffer;
            loop {
                buffer = vec![];
                log::trace!("Waiting for input");
                if let Err(network_error) = stream.recv(&mut buffer) {
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
                    }
                    MessageToServer::Draw1 | MessageToServer::Draw2 | MessageToServer::Draw3 => {
                        let cards_to_draw = msg as u8 - 199;
                        log::log!("Drawing {} cards", cards_to_draw);
                        let mut cards = cards.lock();
                        let cards_drew = cards.draw(cards_to_draw as usize).unwrap_or_else(|| {
                            log::log!("Deck to draw from now empty!");
                            Pile::default()
                        });

                        let mut vec = vec![MessageToClient::SendingCardsToHand as u8; 1];
                        format!("{}", cards_drew)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));

                        stream.send(&vec).unwrap_or_else(|err| {
                            log::error!("Error sending cards to client: {}", err);
                            1
                        });
                        log::log!("Sent new cards to client: {:?}", &vec);
                    }
                    MessageToServer::SendCurrentPilePlease => {
                        let pile = pile.lock();

                        let mut vec = vec![MessageToClient::CurrentPileFollows as u8; 1];
                        format!("{}", pile)
                            .as_bytes()
                            .iter()
                            .for_each(|byte| vec.push(*byte));
                        stream.send(&vec).unwrap();

                        log::log!("Sent pile with data {:?}", &vec);
                    }
                    _ => {}
                }
            }
        });
    }
}
