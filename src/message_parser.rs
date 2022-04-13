use derive_try_from_primitive::TryFromPrimitive;

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum MessageToClient {
    ///No Data
    ///
    /// Client should cleanup
    ServerEnd = 0,

    ///Data - The Total GSAs, then a pile of cards using format!("{}", pile)
    ///
    /// The Client should prepare for the game to start, and add the cards to their private hand
    GameStarting = 10,
    ///Data - a pile of cards using format!("{}", pile)
    ///
    /// The Client should add these cards to their private hand
    SendingCardsToHand = 11,
    ///Data - a bool
    ///
    /// The Client should update the user interface start button - show it if it hasn't started, hide it if it has
    GameHasStartedState = 12,

    ///Data - a pile of cards using format!("{}", pile)
    ///
    /// The Client should update their user interface to show the new dealer pile
    CurrentPileFollows = 20,
    ///Data - a usize in Little Endian Bytes
    ///
    /// The Client should update their user interface to show how many cards are in the dealer pile.
    PileLengthFollows = 21,

    ///Data - a u8 detailing which GSAs we can do. Look at the binary reprs - the first bit signifies the first GSA etc.
    ///
    /// Client should update their user interface to signal that do the GAs
    GsaConditionsFufilled = 30,
}

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum MessageToServer {
    ///No data
    ///
    /// Tick message to server to allow recv to continue
    Tick = 0,
    ///No data
    ///
    ///Server should perform disconnect operations for that client
    Disconnect = 1,
    ///No Data
    ///
    /// Server should send the current dealer pile/length to the client
    SendCurrentPilePlease = 2,
    ///No Data
    ///
    /// Server should attempt to start the game
    ReadyToPlay = 3,
    ///No Data
    ///
    /// Server should send as a bool whether or not the game has started
    HasGameStarted = 4,
    ///No Data
    ///
    /// Server should send a u8 as to which GSAs have been fufilled - using bit reprs
    GsasFufilled = 5,

    //GSAs: All have specific data requirements, which are detailed in their Game impls
    GameAction1 = 20,
    GameAction2 = 21,
    GameAction3 = 22,
    GameAction4 = 23,
    GameAction5 = 24,
}
