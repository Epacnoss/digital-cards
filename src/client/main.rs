use bevy::prelude::{App, ClearColor, Color, DefaultPlugins, Msaa};
use bevy_egui::EguiPlugin;
use cardpack::Pile;
use crossbeam::channel::unbounded;
use digital_cards::{game_type::GSADataData, get_ip, parse_pile, MessageToClient, MessageToServer, TPS_TIMER};
use either::Either;
use parking_lot::Mutex;
use std::{
    convert::TryInto,
    io::{Read, Write},
    net::TcpStream,
    sync::Arc,
    time::{Duration, Instant},
};
use window::{ui_system, MessageToProcessingThread, UiState};

mod window;


fn main() {
    let (to_process_from_ui_tx, to_process_from_ui_rx) = unbounded();

    let hand: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));
    let dealer_pile: Arc<Mutex<Either<Pile, usize>>> =
        Arc::new(Mutex::new(Either::Left(Pile::default())));
    let game_started: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let gsas_fufilled: Arc<Mutex<u8>> = Arc::new(Mutex::new(0));
    let gsas_tot: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));

    //processing thread
    let (
        processing_hand,
        processing_dealer,
        processing_game_started,
        processing_gsas,
        processing_gsas_tot,
    ) = (
        hand.clone(),
        dealer_pile.clone(),
        game_started.clone(),
        gsas_fufilled.clone(),
        gsas_tot.clone(),
    );
    std::thread::spawn(move || {
        println!("Starting thread");
        let mut stream = TcpStream::connect(get_ip()).unwrap_or_else(|err| {
            eprintln!("Error connecting to server: {}", err);
            std::process::exit(1);
        });

        stream
            .set_read_timeout(Some(Duration::from_millis(TPS_TIMER)))
            .unwrap();
        let mut buffer;

        let hand = processing_hand;
        let mut last_tick = Instant::now();
        let mut last_gsa = Instant::now();
        let tps_duration = Duration::from_millis(TPS_TIMER);
        let gsas_duration = Duration::from_millis(TPS_TIMER * 2);

        loop {
            buffer = vec![];

            stream.read(&mut buffer).unwrap(); //TODO: Handle read_exact somehow (maybe agree to send a certain no of packets, but might be wasteful to send 5kb/second)

            if !buffer.is_empty() {
                let msg: MessageToClient = buffer.remove(0).try_into().unwrap();

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
                    stream.write_all(&vec).unwrap();
                };

                match msg {
                    MessageToProcessingThread::Start => {
                        println!("PRO: Start");
                        stream
                            .write_all(&[MessageToServer::ReadyToPlay as u8])
                            .unwrap();
                        print!("PRO: done reading");
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
                // stream.write(&[MessageToServer::Tick as u8]).unwrap()
                stream
                    .write_all(&[MessageToServer::SendCurrentPilePlease as u8])
                    .unwrap();
                last_tick = Instant::now();

                if !*processing_game_started.lock() {
                    stream
                        .write_all(&[MessageToServer::HasGameStarted as u8])
                        .unwrap();
                }
            }
            if last_gsa.elapsed() >= gsas_duration {
                stream
                    .write_all(&[MessageToServer::GsasFufilled as u8])
                    .unwrap();
                last_gsa = Instant::now();
            }
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
