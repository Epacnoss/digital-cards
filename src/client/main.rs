use networking::syncronous::{SyncHost, SyncStream};
// use std::{thread, time::Duration};
use bevy::prelude::{App, ClearColor, Color, DefaultPlugins, Msaa};
use bevy_egui::EguiPlugin;
use cardpack::Pile;
use crossbeam::channel::unbounded;
use digital_cards::{parse_pile, test_config_peer, MessageToClient, MessageToServer};
use networking::error::NetworkError;
use parking_lot::Mutex;
use std::{
    convert::TryInto,
    sync::Arc,
    time::{Duration, Instant},
};
use window::{ui_system, MessageToProcessingThread, UiState};

mod window;

const TPS_TIMER: u64 = 50; // Update ticks 20 times per second

fn main() {
    let (to_process_from_stream_tx, to_process_from_stream_rx) = unbounded();
    let (to_process_from_ui_tx, to_process_from_ui_rx) = unbounded();

    //Config represents this client, peer represents the server
    let (peer, config) = test_config_peer();
    // let (peer, config) = test_config_peer(Layer3Addr::newv4(127, 0, 0, 1), false); 81.151.40.2
    let host = SyncHost::from_host_data(&config).unwrap();

    let stream: Arc<Mutex<SyncStream>> = Arc::new(Mutex::new(host.connect(peer.clone()).unwrap()));
    let hand: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));
    let dealer_pile: Arc<Mutex<Pile>> = Arc::new(Mutex::new(Pile::default()));

    //processing thread
    let (processing_stream, processing_hand, processing_dealer) =
        (stream, hand.clone(), dealer_pile.clone());
    std::thread::spawn(move || {
        let hand = processing_hand;
        let mut last_tick = Instant::now();
        let tps_duration = Duration::from_millis(TPS_TIMER);

        loop {
            for (msg, buffer) in to_process_from_stream_rx.try_iter() {
                println!("PRO: Received message + buffer: {:?}: {:?}", &msg, &buffer);
                match msg {
                    MessageToClient::ServerEnd => {
                        eprintln!("PRO: Server disconnected!");
                        std::process::exit(0);
                    }
                    MessageToClient::SendingCardsToHand => {
                        println!("PRO: Receiving cards: {:?}", &buffer);
                        let string = String::from_utf8(buffer).unwrap();

                        let mut hand = hand.lock();
                        for card in parse_pile(string) {
                            hand.push(card);
                        }
                    }
                    MessageToClient::CurrentPileFollows => {
                        let string = String::from_utf8(buffer).unwrap();
                        let pile = Pile::from_vector(parse_pile(string));
                        *processing_dealer.lock() = pile;
                    }
                    _ => {}
                }
            }

            let mut stream = processing_stream.lock();

            for msg in to_process_from_ui_rx.try_iter() {
                println!("PRO: Received msg from UI: {:?}.", msg);

                match msg {
                    MessageToProcessingThread::Draw1 => {
                        stream.send(&[MessageToServer::Draw1 as u8; 1]).unwrap();
                    }
                    MessageToProcessingThread::Draw2 => {
                        stream.send(&[MessageToServer::Draw2 as u8; 1]).unwrap();
                    }
                    MessageToProcessingThread::Draw3 => {
                        stream.send(&[MessageToServer::Draw3 as u8; 1]).unwrap();
                    }
                    MessageToProcessingThread::SendHandToPile => {
                        let hand: Vec<u8> = {
                            let mut hand = hand.lock();
                            let ctd = hand.cards().len();
                            format!("{}", hand.draw(ctd).unwrap_or_default())
                                .as_bytes()
                                .to_vec()
                        };

                        let mut vec = vec![MessageToServer::AddingToPile as u8; 1];
                        hand.into_iter().for_each(|b| vec.push(b));

                        stream.send(&vec).unwrap();
                    }
                    MessageToProcessingThread::SendSpecificCardsToPile(pile) => {
                        let hand: Vec<u8> = format!("{}", pile).as_bytes().to_vec();

                        let mut vec = vec![MessageToServer::AddingToPile as u8; 1];
                        hand.into_iter().for_each(|b| vec.push(b));

                        stream.send(&vec).unwrap();
                    }
                }
            }

            if last_tick.elapsed() >= tps_duration {
                stream.send(&[MessageToServer::Tick as u8; 1]).iter();
                last_tick = Instant::now();
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
            })
            .add_plugins(DefaultPlugins)
            .add_plugin(EguiPlugin)
            .add_system(ui_system)
            .run();
    }
}
