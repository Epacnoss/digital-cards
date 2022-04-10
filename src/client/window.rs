use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use cardpack::Pile;
use crossbeam::channel::Sender;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug)]
pub struct UiState {
    pub hand: Arc<Mutex<Pile>>,
    pub dealer: Arc<Mutex<Pile>>,
    pub tx: Sender<MessageToProcessingThread>,
}

#[non_exhaustive]
#[derive(Copy, Clone, Debug)]
pub enum MessageToProcessingThread {
    Draw1,
    Draw2,
    Draw3,
    SendHandToPile,
}

pub fn ui_system(egui_ctx: Res<EguiContext>, ui_state: Res<UiState>) {
    egui::panel::SidePanel::left("lhs").show(egui_ctx.ctx(), |ui| {
        ui.heading("Digital Cards");

        ui.separator();
        ui.heading("Current Hand: ");
        ui.label(format!("{}", ui_state.hand.lock()));

        ui.separator();
        ui.heading("Current Dealer Pile: ");
        ui.label(format!("{}", ui_state.dealer.lock()));
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
        if ui.button("Send Hand to Pile").clicked() {
            ui_state
                .tx
                .send(MessageToProcessingThread::SendHandToPile)
                .unwrap();
        }
    });
}
