
use std::sync::mpsc::{Receiver,  Sender};
use rand::seq::IteratorRandom;
use rand::thread_rng;
use brydz_core::deal::{DealMaintainer};
use brydz_core::error::{BridgeErrorStd,  Mismatch};
use brydz_core::error::HandError::EmptyHand;
use brydz_core::player::side::Side;
use brydz_core::player::situation::Situation;
use brydz_core::protocol::{ClientDealMessage,  DealAction,  ServerDealMessage};
use brydz_core::protocol::DealAction::{PlayCard};
use brydz_core::error::DealError::DealFull;
use brydz_core::error::TrickError::ViolatedOrder;
use brydz_core::world::agent::{Agent, AwareAgent, CommunicatingAgent};

pub struct DeclarerOverChannel{
    sender: Sender<ClientDealMessage>,
    receiver: Receiver<ServerDealMessage>,
    situation: Situation,
}

impl DeclarerOverChannel{
    pub fn new(sender: Sender<ClientDealMessage>, receiver: Receiver<ServerDealMessage>, situation: Situation) -> Self{
        Self{sender, receiver, situation}
    }

    pub fn side(&self) -> Side{
        self.situation.side()
    }
    pub fn partner_side(&self) -> Side{
        self.situation.side().partner()
    }
    pub fn current_side(&self) -> Option<Side>{
        self.situation.current_side()
    }


}

impl Agent<DealAction> for DeclarerOverChannel {
    fn select_action(&self) -> Result<DealAction, BridgeErrorStd> {
        let mut rng = thread_rng();
        match self.situation.current_side(){
            None => Err(DealFull.into()),
            Some(my) if my == self.situation.side()  => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_hand().iter().choose(&mut rng)
                        .map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |cr|  Ok(PlayCard(cr.to_owned()))),
                    Some(s) => match self.situation.hand().cards_in_suit(s).iter().choose(&mut rng){
                        None => self.situation.cards_hand().iter().choose(&mut rng).map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |c| Ok(PlayCard(c.to_owned()))),
                        Some(c) => Ok(PlayCard(c.to_owned()))
                    }
                }

            },
            Some(dummy) if  dummy == self.situation.side().partner() => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_dummy().iter().choose(&mut rng)
                        .map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |cr|  Ok(PlayCard(cr.to_owned()))),
                    Some(s) => match self.situation.dummy_hand().cards_in_suit(s).iter().choose(&mut rng){
                        None => self.situation.cards_dummy().iter().choose(&mut rng).map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |c| Ok(PlayCard(c.to_owned()))),
                        Some(c) => Ok(PlayCard(c.to_owned()))
                    }
                }

            }
            Some(bad) => Err(ViolatedOrder(Mismatch{expected: bad, found: self.side()}).into())
        }
    }
}
impl AwareAgent<Situation> for DeclarerOverChannel{
    fn env(&self) -> &Situation {
        &self.situation
    }

    fn env_mut(&mut self) -> &mut Situation {
        &mut self.situation
    }
}

impl CommunicatingAgent<ServerDealMessage, ClientDealMessage, DealAction, BridgeErrorStd> for DeclarerOverChannel{
    fn send(&self, message: ClientDealMessage) -> Result<(), BridgeErrorStd> {
        self.sender.send(message).map_err(|e| e.into())
    }

    fn recv(&self) -> Result<ServerDealMessage, BridgeErrorStd> {
        self.receiver.recv().map_err(|e| e.into())
    }
}