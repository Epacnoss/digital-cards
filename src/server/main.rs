use cardpack::Pile;
use crossbeam::channel::unbounded;
use digital_cards::{
    cheat::Cheat,
    game_type::{GSAResult, Game, GamePlaying},
    mpmc::BroadcastChannel,
    parse_pile, test_config, MessageToClient, MessageToServer,
};
use networking::{
    error::NetworkError,
    syncronous::{SyncDataStream, SyncHost},
    ConnectionRequest,
};
use std::{convert::TryInto, sync::Arc};

fn main() {
    pretty_logger::init_to_defaults().unwrap();

    let config = test_config(true, false);
    let host = SyncHost::from_host_data(&config).unwrap();

    let game = Arc::new(GamePlaying::<Cheat>::default());
    let pile = game.arc_dealer_pile();

    let (streams_tx, streams_rx) = unbounded();
    let general_broadcast_channel = Arc::new(BroadcastChannel::new());
    let game_broadcast_channel = Arc::new(BroadcastChannel::new());

    std::thread::spawn(move || {
        let mut streams_buffer = None;
        for netstream in host {
            // let stream = netstream.unwrap().verify(&peer.clone()).unwrap();
            //TODO: Whitelist mode
            let stream = unsafe { netstream.unwrap().unverify() };

            if let Some(buf_stream) = std::mem::take(&mut streams_buffer) {
                streams_tx.send((buf_stream, stream)).unwrap();
            } else {
                streams_buffer = Some(stream);
            }
        }
    });

    for (mut processing_stream, mut recv_stream) in streams_rx.iter() {
        let pile = pile.clone();
        let general_bc = general_broadcast_channel.clone();
        let game_bc = game_broadcast_channel.clone();
        let game = game.clone();

        std::thread::spawn(move || {
            log::info!("new connection from {:?}", processing_stream.addr());
            let (general_bc_id, game_bc_id) = (general_bc.subscribe(), game_bc.subscribe());
            let game_id = game.subscribe().unwrap();

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
                    game_bc.unsubscribe(game_id);
                    return;
                }

                let msg: MessageToServer = buffer.remove(0).try_into().unwrap();
                if msg != MessageToServer::Tick
                    && msg != MessageToServer::HasGameStarted
                    && msg != MessageToServer::SendCurrentPilePlease
                {
                    log::info!("Client sent message: {:?} with data {:?}", &msg, &buffer);
                }

                //TODO: Make it for not just piles
                //Use the GSADataTaken
                let match_on_gsa_stuff = |gsa: GSAResult| match gsa {
                    GSAResult::PlayerTakesAllCards(pile, id) => {
                        let mut vec = vec![Pile::default(); game_bc.num_subscribed()];
                        vec[id] = pile;
                        game_bc.send((vec, false)).unwrap();
                    }
                    _ => {}
                };

                match msg {
                    MessageToServer::SendCurrentPilePlease => {
                        let pile = pile.lock();

                        if game.hidden_pile_self() {
                            let mut vec = vec![MessageToClient::PileLengthFollows as u8; 1];
                            pile.len()
                                .to_le_bytes()
                                .into_iter()
                                .for_each(|b| vec.push(b));
                            recv_stream.send(&vec).unwrap();
                        } else {
                            let mut vec = vec![MessageToClient::CurrentPileFollows as u8; 1];
                            vec.append(&mut format!("{}", pile).as_bytes().to_vec());
                            recv_stream.send(&vec).unwrap();
                        }
                    }
                    MessageToServer::ReadyToPlay => {
                        if let Some(new_pile) = game.start(game_bc.clone()) {
                            *pile.lock() = new_pile;
                        }
                    }
                    MessageToServer::Disconnect => {
                        log::info!("Client disconnected!");
                        return;
                    }
                    MessageToServer::HasGameStarted => {
                        recv_stream
                            .send(&[
                                MessageToClient::GameHasStartedState as u8,
                                game.has_started() as u8,
                            ])
                            .unwrap();
                    }
                    MessageToServer::GsasFufilled => {
                        recv_stream
                            .send(&[
                                MessageToClient::GsaConditionsFufilled as u8,
                                game.gsas_fufilled(game_id),
                            ])
                            .unwrap();
                    }
                    MessageToServer::GameAction1 => {
                        match_on_gsa_stuff(game.gsa_1(
                            game_id,
                            Pile::from_vector(parse_pile(String::from_utf8(buffer).unwrap())),
                        ));
                    }
                    MessageToServer::GameAction2 => {
                        match_on_gsa_stuff(game.gsa_2(
                            game_id,
                            Pile::from_vector(parse_pile(String::from_utf8(buffer).unwrap())),
                        ));
                    }
                    MessageToServer::GameAction3 => {
                        match_on_gsa_stuff(game.gsa_3(game_id, ()));
                    }
                    MessageToServer::GameAction4 => {
                        match_on_gsa_stuff(game.gsa_4(game_id, ()));
                    }
                    MessageToServer::GameAction5 => {
                        match_on_gsa_stuff(game.gsa_5(game_id, ()));
                    }
                    _ => {}
                }

                for msg in general_bc.receive(general_bc_id) {
                    log::info!("BC Received msg: {:?}", msg);
                    match msg {
                        ServerMessage::UpdateDealerPile => {
                            let pile = pile.lock();
                            let mut vec = vec![MessageToClient::CurrentPileFollows as u8; 1];
                            vec.append(&mut format!("{}", pile).as_bytes().to_vec());
                            recv_stream.send(&vec).unwrap();
                        }
                    }
                }
                for (mut buffer, is_start) in game_bc.receive(game_bc_id) {
                    log::info!("GBC received pile");
                    let mut vec = if is_start {
                        let mut v = vec![MessageToClient::GameStarting as u8];
                        v.append(&mut game.gsa_number().to_le_bytes().to_vec());
                        v
                    } else {
                        vec![MessageToClient::SendingCardsToHand as u8]
                    };
                    vec.append(&mut format!("{}", buffer.remove(game_id)).as_bytes().to_vec());
                    recv_stream.send(&vec).unwrap();
                }
            }
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ServerMessage {
    UpdateDealerPile,
}
