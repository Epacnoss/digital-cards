use derive_try_from_primitive::TryFromPrimitive;

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum MessageToClient {
    ServerEnd = 0,
    SendingCardsToHand = 1,
    CurrentPileFollows = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, TryFromPrimitive, Eq, PartialEq, Debug)]
#[non_exhaustive]
pub enum MessageToServer {
    Tick = 0,
    Disconnect = 1,
    AddingToPile = 2,
    SendCurrentPilePlease = 3,
    Draw1 = 200,
    Draw2 = 201,
    Draw3 = 202,
}
