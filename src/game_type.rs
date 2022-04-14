use crate::mpmc::BroadcastChannel;
use cardpack::Pile;
use parking_lot::Mutex;
use std::{fmt::Debug, ops::Deref, sync::Arc};

#[derive(Default, Debug)]
pub struct GamePlaying<T: Game>(pub T);

impl<T: Game> Deref for GamePlaying<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

///Trait for a Game Type
pub trait Game: Default + Debug {
    const MIN_PLAYERS: usize;
    const GSAS: usize;
    const DEALER_PILE_HIDDEN: bool = false;

    type GSA1Params;
    type GSA2Params;
    type GSA3Params;
    type GSA4Params;
    type GSA5Params;

    ///Subscribes a client and provides their ID.
    /// Will return None is game is currently in progress
    fn subscribe(&self) -> Option<usize>;
    ///Prepares to start a game. Will only start if there are enough clients, and the game isn't already in progress
    ///
    /// If there are enough clients, it sends a hand to all of them.
    fn start(&self, broadcast_channel: Arc<BroadcastChannel<(Vec<Pile>, bool)>>) -> Option<Pile>;

    ///Returns an Arc to the dealer pile
    fn arc_dealer_pile(&self) -> Arc<Mutex<Pile>>;
    ///Returns whether or not the dealer pile should be hidden to the client(s)
    fn hidden_pile_self(&self) -> bool {
        Self::DEALER_PILE_HIDDEN
    }
    ///Communicates to Server whether or not the game has been started yet.
    fn has_started(&self) -> bool;
    ///Guaranteed to have len `Self::GSAS`. Returns an array of GSA names, and whether or not the client should remove cards from the pile when triggering them
    fn gsa_names(&self) -> &'static [(&'static str, GSADataTaken)] {
        Self::gsa_names_static()
    }
    ///Guaranteed to have len `Self::GSAS`. Returns an array of GSA names, and whether or not the client should remove cards from the pile when triggering them
    fn gsa_names_static() -> &'static [(&'static str, GSADataTaken)];
    fn gsa_number(&self) -> usize {
        Self::GSAS
    }
    fn last_player_id(&self) -> usize;

    #[must_use]
    fn gsa_1(&self, _caller_id: usize, _: Self::GSA1Params) -> GSAResult {
        GSAResult::default()
    }
    #[must_use]
    fn gsa_2(&self, _caller_id: usize, _: Self::GSA2Params) -> GSAResult {
        GSAResult::default()
    }
    #[must_use]
    fn gsa_3(&self, _caller_id: usize, _: Self::GSA3Params) -> GSAResult {
        GSAResult::default()
    }
    #[must_use]
    fn gsa_4(&self, _caller_id: usize, _: Self::GSA4Params) -> GSAResult {
        GSAResult::default()
    }
    #[must_use]
    fn gsa_5(&self, _caller_id: usize, _: Self::GSA5Params) -> GSAResult {
        GSAResult::default()
    }

    fn gsas_fufilled(&self, caller_id: usize) -> u8;
}

#[derive(Copy, Clone, Debug)]
pub enum GSADataTaken {
    ShowCards,
    TakeCards,
    Nothing,
}
#[derive(Clone, Debug)]
pub enum GSADataData {
    ShowCards(Pile),
    TakeCards(Pile),
    Nothing,
}
#[derive(Clone, Debug)]
pub enum GSAResult {
    PlayerTakesAllCards(Pile, usize),
    Nothing,
}
impl Default for GSAResult {
    fn default() -> Self {
        Self::Nothing
    }
}
