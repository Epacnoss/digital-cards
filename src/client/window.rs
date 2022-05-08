use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use cardpack::{Card, Pile};
use crossbeam::channel::Sender;
use digital_cards::{
    cheat::Cheat,
    game_type::{GSADataData, GSADataTaken, Game},
};
use either::Either;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub struct UiState {
    pub hand: Arc<Mutex<Pile>>,
    pub dealer: Arc<Mutex<Either<Pile, usize>>>,
    pub game_started: Arc<Mutex<bool>>,
    pub gsas_fufilled: Arc<Mutex<u8>>,
    pub gsas: Arc<Mutex<usize>>,
    pub tx: Sender<MessageToProcessingThread>,
    pub checked: Vec<bool>,
    pub old_cards: Vec<Card>,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum MessageToProcessingThread {
    Start,
    GSA1(GSADataData),
    GSA2(GSADataData),
    GSA3(GSADataData),
    GSA4(GSADataData),
    GSA5(GSADataData),
}

#[allow(clippy::collapsible_if)]
pub fn ui_system(egui_ctx: Res<EguiContext>, mut ui_state: ResMut<UiState>) {
    let hand_vec = ui_state.hand.lock().cards().clone();
    let gsas = *ui_state.gsas_fufilled.lock();

    if hand_vec != ui_state.old_cards {
        ui_state.checked = vec![false; hand_vec.len()];
        ui_state.old_cards = hand_vec.clone();
    }

    egui::panel::SidePanel::left("lhs").show(egui_ctx.ctx(), |ui| {
        ui.heading("Digital Cards");

        ui.separator();
        ui.heading("Current Hand: ");
        for (i, card) in hand_vec.clone().into_iter().enumerate() {
            if gsas == 0 {
                ui.label(format_card(&card));
            } else {
                ui.checkbox(ui_state.checked.get_mut(i).unwrap(), format_card(&card));
            }
        }

        ui.separator();
        ui.heading("Current Dealer Pile: ");
        let current = ui_state.dealer.lock();
        ui.label(match current.clone() {
            Either::Left(pile) => format!(
                "Currently has: {}",
                format_pile(pile).into_iter().collect::<String>()
            ),
            Either::Right(size) => format!("Currently has {} cards", size),
        });
        println!("UI: Current DP: {:?}", current);
    });

    egui::panel::SidePanel::right("rhs").show(egui_ctx.ctx(), |ui| {
        ui.heading("Buttons!");

        ui.separator();

        if !*ui_state.game_started.lock() {
            if ui.button("Start Game!").clicked() {
                ui_state.tx.send(MessageToProcessingThread::Start).unwrap();
            }
        }

        if gsas != 0 {
            for (i, (gsa_title, remove_cards_from_hand)) in (0..*ui_state.gsas.lock())
                .into_iter()
                .zip(Cheat::gsa_names_static().iter().copied())
            //TODO: not hardocde this
            {
                let mut hand = hand_vec
                    .clone()
                    .into_iter()
                    .map(Option::Some)
                    .collect::<Vec<_>>();
                let mut being_sent = Pile::default();
                for (i, card_opt) in hand.iter_mut().enumerate() {
                    if ui_state.checked[i] {
                        being_sent.push(std::mem::take(card_opt).unwrap());
                    }
                }

                let send_msg = |conv: &dyn Fn(GSADataData) -> MessageToProcessingThread| {
                    let msg = match remove_cards_from_hand {
                        GSADataTaken::ShowCards => conv(GSADataData::ShowCards(being_sent.clone())),
                        GSADataTaken::TakeCards => {
                            *ui_state.hand.lock() = hand.into_iter().flatten().collect();
                            conv(GSADataData::ShowCards(being_sent.clone()))
                        }
                        _ => conv(GSADataData::Nothing),
                    };
                    ui_state.tx.send(msg).unwrap();
                };

                match i {
                    0 => {
                        if gsas & 0b1000_0000 >= 1 {
                            if ui.button(gsa_title).clicked() {
                                send_msg(&|data| MessageToProcessingThread::GSA1(data));
                            }
                        }
                    }
                    1 => {
                        if gsas & 0b0100_0000 >= 1 {
                            if ui.button(gsa_title).clicked() {
                                send_msg(&|data| MessageToProcessingThread::GSA2(data));
                            }
                        }
                    }
                    2 => {
                        if gsas & 0b0010_0000 >= 1 {
                            if ui.button(gsa_title).clicked() {
                                send_msg(&|data| MessageToProcessingThread::GSA3(data));
                            }
                        }
                    }
                    3 => {
                        if gsas & 0b0001_0000 >= 1 {
                            if ui.button(gsa_title).clicked() {
                                send_msg(&|data| MessageToProcessingThread::GSA4(data));
                            }
                        }
                    }
                    4 => {
                        if gsas & 0b0000_1000 >= 1 {
                            if ui.button(gsa_title).clicked() {
                                send_msg(&|data| MessageToProcessingThread::GSA5(data));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });
}

pub fn format_pile(p: Pile) -> Vec<String> {
    p.cards()
        .iter()
        .map(|card| format!("{}\n", format_card(card)))
        .collect()
}

pub fn format_card(card: &Card) -> String {
    use cardpack::{CLUBS, DIAMONDS, HEARTS, SPADES};

    let rank = format!("{}", card.rank);
    let alt_rank = match rank.as_str() {
        "J" => Some("Jack"),
        "Q" => Some("Queen"),
        "K" => Some("King"),
        "A" => Some("Ace"),
        "T" => Some("10"),
        _ => None,
    };
    let rank = alt_rank.unwrap_or(rank.as_str());

    let suit = match format!("{}", card.suit.name).as_str() {
        // SPADES => "♤",
        // DIAMONDS => "♢",
        // HEARTS => "♡",
        // CLUBS => "♧",
        SPADES => "Spades",
        DIAMONDS => "Diamonds",
        HEARTS => "Hearts",
        CLUBS => "Clubs",
        _ => "",
    };

    format!("{} of {}", rank, suit)
}
