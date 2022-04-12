use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use cardpack::{Card, Pile};
use crossbeam::channel::Sender;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub struct UiState {
    pub hand: Arc<Mutex<Pile>>,
    pub dealer: Arc<Mutex<Pile>>,
    pub tx: Sender<MessageToProcessingThread>,
    pub checked: Vec<bool>,
    pub old_cards: Vec<Card>,
}

#[non_exhaustive]
#[derive(Clone, Debug)]
pub enum MessageToProcessingThread {
    Draw1,
    Draw2,
    Draw3,
    SendHandToPile,
    SendSpecificCardsToPile(Pile),
}

pub fn ui_system(egui_ctx: Res<EguiContext>, mut ui_state: ResMut<UiState>) {
    let hand_vec = ui_state.hand.lock().cards().clone();
    if hand_vec != ui_state.old_cards {
        ui_state.checked = vec![false; hand_vec.len()];
        ui_state.old_cards = hand_vec.clone();
    }

    egui::panel::SidePanel::left("lhs").show(egui_ctx.ctx(), |ui| {
        ui.heading("Digital Cards");

        ui.separator();
        ui.heading("Current Hand: ");
        // ui.label(format!("{:?}", format_pile(ui_state.hand.lock().clone())));
        for (i, card) in hand_vec.clone().into_iter().enumerate() {
            ui.checkbox(ui_state.checked.get_mut(i).unwrap(), format_card(&card));
        }

        ui.separator();
        ui.heading("Current Dealer Pile: ");
        ui.label(
            format_pile(ui_state.dealer.lock().clone())
                .into_iter()
                .collect::<String>(),
        );
    });

    egui::panel::SidePanel::right("rhs").show(egui_ctx.ctx(), |ui| {
        ui.heading("Buttons!");

        ui.separator();

        if ui.button("Draw 1").clicked() {
            ui_state.tx.send(MessageToProcessingThread::Draw1).unwrap();
        }
        if ui.button("Draw 2").clicked() {
            ui_state.tx.send(MessageToProcessingThread::Draw2).unwrap();
        }
        if ui.button("Draw 3").clicked() {
            ui_state.tx.send(MessageToProcessingThread::Draw3).unwrap();
        }

        ui.separator();
        if ui.button("Send selected cards to Pile").clicked() {
            let mut hand = hand_vec
                .clone()
                .into_iter()
                .map(|card| Option::Some(card))
                .collect::<Vec<_>>();
            let mut being_sent = Pile::default();
            for (i, card_opt) in hand.iter_mut().enumerate() {
                if ui_state.checked[i] {
                    being_sent.push(std::mem::take(card_opt).unwrap());
                }
            }

            ui_state
                .tx
                .send(MessageToProcessingThread::SendSpecificCardsToPile(
                    being_sent,
                ))
                .unwrap();
            *ui_state.hand.lock() = hand.into_iter().flatten().collect();
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
