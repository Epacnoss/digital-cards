use bevy::prelude::{App, ClearColor, Color, DefaultPlugins, Msaa};
use bevy_egui::EguiPlugin;
use cardpack::Pile;
use crossbeam::channel::unbounded;
use digital_cards::{
    game_type::GSADataData, parse_pile, test_config_peer, MessageToClient, MessageToServer,
};
use either::Either;
use networking::{
    error::NetworkError,
    syncronous::{SyncHost, SyncStream},
};
use parking_lot::Mutex;
use std::{
    convert::TryInto,
    sync::Arc,
    time::{Duration, Instant},
};
use window::{ui_system, MessageToProcessingThread, UiState};

mod window;

const TPS_TIMER: u64 = 100;

fn main() {
    let (to_process_from_stream_tx, to_process_from_stream_rx) = unbounded();
    let (to_process_from_ui_tx, to_process_from_ui_rx) = unbounded();

    let local_server = true;
    //Config represents this client, peer represents the server
    let (peer, config) = test_config_peer(local_server);
    let host = if local_server {
        SyncHost::client_only(&config).unwrap()
    } else {
        SyncHost::from_host_data(&config).unwrap()
    };

    let stream: Arc<Mutex<SyncStream>> = Arc::new(Mutex::new(host.connect(peer.clone()).unwrap()));
    let hand: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));
    let dealer_pile: Arc<Mutex<Either<Pile, usize>>> =
        Arc::new(Mutex::new(Either::Left(Pile::default())));
    let game_started: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let gsas_fufilled: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let gsas_tot: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    //processing thread
    let (
        processing_stream,
        processing_hand,
        processing_dealer,
        processing_game_started,
        processing_gsas,
        processing_gsas_tot,
    ) = (
        stream,
        hand.clone(),
        dealer_pile.clone(),
        game_started.clone(),
        gsas_fufilled.clone(),
        gsas_tot.clone(),
    );
    std::thread::spawn(move || {
        let hand = processing_hand;
        let mut last_tick = Instant::now();
        let mut last_gsa = Instant::now();
        let tps_duration = Duration::from_millis(TPS_TIMER);
        let gsas_duration = Duration::from_millis(TPS_TIMER * 2);

        loop {
            for (msg, buffer) in to_process_from_stream_rx.try_iter() {
                let mut buffer: Vec<u8> = buffer;
                println!("PRO: Received message + buffer: {:?}: {:?}", &msg, &buffer);
                match msg {
                    MessageToClient::GameStarting => {
                        println!("Game Starting!");
                        *processing_gsas_tot.lock() = {
                            let mut arr = [0_u8; 8];
                            for a in arr.iter_mut() {
                                *a = buffer.remove(0);
                            }
                            usize::from_le_bytes(arr)
                        };
                        println!("Total GSAs: {}", *processing_gsas_tot.lock());
                        *processing_game_started.lock() = true;

                        hand.lock().append(&Pile::from_vector(parse_pile(
                            String::from_utf8(buffer).unwrap(),
                        )));
                    }
                    MessageToClient::ServerEnd => {
                        eprintln!("PRO: Server disconnected!");
                        std::process::exit(0);
                    }
                    MessageToClient::SendingCardsToHand => {
                        println!("PRO: Receiving cards: {:?}", &buffer);
                        hand.lock().append(&Pile::from_vector(parse_pile(
                            String::from_utf8(buffer).unwrap(),
                        )));
                    }
                    MessageToClient::CurrentPileFollows => {
                        let pile =
                            Pile::from_vector(parse_pile(String::from_utf8(buffer).unwrap()));
                        *processing_dealer.lock() = Either::Left(pile);
                        println!("Updated pile");
                    }
                    MessageToClient::PileLengthFollows => {
                        let mut arr = [0_u8; 8];
                        for a in arr.iter_mut() {
                            *a = buffer.remove(0);
                        }

                        *processing_dealer.lock() = Either::Right(usize::from_le_bytes(arr));
                        println!("Updated pile length");
                    }
                    MessageToClient::GameHasStartedState => {
                        *processing_game_started.lock() = buffer.remove(0) != 0;
                    }
                    MessageToClient::GsaConditionsFufilled => {
                        *processing_gsas.lock() = buffer.remove(0)
                    }
                    _ => {}
                }
            }

            let mut stream = processing_stream.lock();

            for msg in to_process_from_ui_rx.try_iter() {
                println!("PRO: Received msg from UI: {:?}", msg);

                let mut match_on_gsa_data = |gsa_data: GSADataData, msg: MessageToServer| {
                    let mut vec = vec![msg as u8];
                    match gsa_data {
                        GSADataData::ShowCards(pile) | GSADataData::TakeCards(pile) => {
                            vec.append(&mut format!("{}", pile).as_bytes().to_vec());
                        }
                        GSADataData::Nothing => {}
                    }
                    stream.send(&vec).unwrap();
                };

                match msg {
                    MessageToProcessingThread::Start => {
                        println!("PRO: Start");
                        stream.send(&[MessageToServer::ReadyToPlay as u8]).unwrap();
                    }
                    MessageToProcessingThread::GSA1(gsa_data) => {
                        match_on_gsa_data(gsa_data, MessageToServer::GameAction1);
                    }
                    MessageToProcessingThread::GSA2(gsa_data) => {
                        match_on_gsa_data(gsa_data, MessageToServer::GameAction2);
                    }
                    MessageToProcessingThread::GSA3(gsa_data) => {
                        match_on_gsa_data(gsa_data, MessageToServer::GameAction3);
                    }
                    MessageToProcessingThread::GSA4(gsa_data) => {
                        match_on_gsa_data(gsa_data, MessageToServer::GameAction4);
                    }
                    MessageToProcessingThread::GSA5(gsa_data) => {
                        match_on_gsa_data(gsa_data, MessageToServer::GameAction5);
                    }
                }
            }

            if last_tick.elapsed() >= tps_duration {
                // stream.send(&[MessageToServer::Tick as u8]).unwrap()
                stream
                    .send(&[MessageToServer::SendCurrentPilePlease as u8])
                    .unwrap();
                last_tick = Instant::now();

                if !*processing_game_started.lock() {
                    stream
                        .send(&[MessageToServer::HasGameStarted as u8])
                        .unwrap();
                }
            }
            if last_gsa.elapsed() >= gsas_duration {
                stream.send(&[MessageToServer::GsasFufilled as u8]).unwrap();
                last_gsa = Instant::now();
            }
        }
    });

    //networking recv thread
    std::thread::spawn(move || {
        let mut buffer;
        println!("NET: Creating Secondary Stream");
        let mut recv_stream = SyncHost::client_only(&config)
            .unwrap()
            .connect(peer.clone())
            .unwrap();
        println!("NET: Ready to Go!");

        loop {
            println!("NET: Start of recv loop");
            buffer = vec![];

            if let Err(network_error) = recv_stream.recv(&mut buffer) {
                match network_error {
                    NetworkError::IOError(io_error) => {
                        eprintln!("IO Error: {}", io_error);
                    }
                    _ => {
                        eprintln!("Other Error: {}", network_error);
                    }
                }
                std::process::exit(1);
            }
            println!("NET: Received data! {:?}", &buffer);

            let msg: MessageToClient = buffer.remove(0).try_into().unwrap();
            println!(
                "NET: Client received message, and sent to channel: {:?}",
                &msg
            );
            to_process_from_stream_tx.send((msg, buffer)).unwrap();
        }
    });

    //render part
    {
        App::new()
            .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
            .insert_resource(Msaa { samples: 4 })
            .insert_resource(UiState {
                hand,
                dealer: dealer_pile,
                tx: to_process_from_ui_tx,
                checked: vec![],
                old_cards: vec![],
                game_started,
                gsas_fufilled,
                gsas: gsas_tot,
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(EguiPlugin)
            .add_system(ui_system)
            .run();
    }
}
