use std::sync::mpsc::{Receiver, Sender};
use bridge_core::error::BridgeErrorStd;
use bridge_core::player::situation::Situation;
use bridge_core::protocol::{ClientMessage, DealNotify, ServerMessage};
use bridge_core::protocol::ClientMessage::Quit;
use bridge_core::protocol::DealAction::ShowHand;
use crate::Bot;
use log::info;
use bridge_core::error::FlowError::ServerDead;

pub struct DummyOverChannel{
    sender: Sender<ClientMessage>,
    receiver: Receiver<ServerMessage>,
    situation: Situation,
}

impl DummyOverChannel{
    pub fn new(sender: Sender<ClientMessage>, receiver: Receiver<ServerMessage>, situation: Situation) -> Self{
        Self{sender, receiver, situation}
    }

}

impl Bot for DummyOverChannel{
    fn run(&mut self) -> Result<(), BridgeErrorStd> {
        self.sender.send(ClientMessage::Ready)?;
        loop{
            match self.receiver.recv()?{
                ServerMessage::Deal(notify) => match notify{
                    DealNotify::YourMove => {
                        self.sender.send(ClientMessage::Dealing(ShowHand(self.situation.hand().clone())))?
                    }
                    DealNotify::DealClosed => {
                        self.sender.send(Quit)?;
                        return Ok(())
                    }
                    _ => {}
                }
                ServerMessage::Bidding(_) => {}
                ServerMessage::PlayerLeft(_) => {}
                ServerMessage::DealInfo(_) => {}
                ServerMessage::BiddingInfo(_) => {}
                ServerMessage::GameOver => {
                    return Ok(())
                }
                ServerMessage::Error(_) => {}
                ServerMessage::ServerNotReady => {}
                ServerMessage::ServerStopping => {
                    info!("Server is stopping. Exiting. Dummy signing out");
                    return Err(ServerDead.into());
                },
            }
        }
    }
}