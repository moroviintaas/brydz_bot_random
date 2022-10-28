use std::sync::mpsc::{Receiver, Sender};
use brydz_core::player::situation::Situation;
use brydz_framework::error::BridgeErrorStd;
use brydz_framework::protocol::{ClientDealMessage, DealNotify, ServerDealMessage};
use crate::Bot;
use brydz_framework::protocol::ClientControlMessage::{IamReady, Quit};
use brydz_framework::protocol::ClientDealInformation::ShowHand;

pub struct DummyOverChannel{
    sender: Sender<ClientDealMessage>,
    receiver: Receiver<ServerDealMessage>,
    situation: Situation,
}

impl DummyOverChannel{
    pub fn new(sender: Sender<ClientDealMessage>, receiver: Receiver<ServerDealMessage>, situation: Situation) -> Self{
        Self{sender, receiver, situation}
    }

}


impl Bot for DummyOverChannel{
    fn run(&mut self) -> Result<(), BridgeErrorStd> {
        self.sender.send(IamReady.into())?;
        loop{
            match self.receiver.recv()?{
                ServerDealMessage::Notify(notify) => match notify{
                    DealNotify::YourMove => {
                        self.sender.send(ShowHand(self.situation.hand().clone()).into())?
                    },
                    DealNotify::DealClosed => {
                        self.sender.send(Quit.into()).unwrap_or(());
                        return Ok(())
                    },
                    _ => {}
                }
                ServerDealMessage::Info(_) => {}
                ServerDealMessage::Control(_) => {}
            }
        }
    }
}