use rand::seq::IteratorRandom;
use rand::thread_rng;
use brydz_core::deal::DealMaintainer;
use brydz_core::distribution::hand::BridgeHand;
use brydz_core::error::{BridgeCoreErrorStd, Mismatch};
use brydz_core::error::DealError::DealFull;
use brydz_core::error::HandError::EmptyHand;
use brydz_core::error::TrickError::ViolatedOrder;
use brydz_core::player::situation::Situation;
use brydz_framework::error::BridgeErrorStd;
use brydz_framework::protocol::{ClientDealMessage, DealAction, ServerDealMessage};
use brydz_framework::protocol::DealAction::PlayCard;
use brydz_framework::world::agent::{Agent,  AwareAgent, CommunicatingAgent};
use brydz_framework::world::comm::{CommunicationEnd};

pub struct DeclarerBot<Comm: CommunicationEnd< ClientDealMessage, ServerDealMessage, BridgeErrorStd>>{
    situation: Situation,
    comm: Comm,
}

impl<Comm> DeclarerBot<Comm>
where Comm: CommunicationEnd< ClientDealMessage, ServerDealMessage, BridgeErrorStd>{
    pub fn new(comm: Comm, situation: Situation) -> Self{
        Self{comm, situation}
    }
}

impl<Comm> Agent<DealAction> for DeclarerBot<Comm>
where Comm: CommunicationEnd<ClientDealMessage, ServerDealMessage, BridgeErrorStd>{
    fn select_action(&self) -> Result<DealAction, BridgeCoreErrorStd> {
        let mut rng = thread_rng();
        match self.situation.current_side(){
            None => Err(DealFull.into()),
            Some(my) if my == self.situation.side()  => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_hand().iter().choose(&mut rng)
                        .map_or(Err(BridgeCoreErrorStd::Hand(EmptyHand)), |cr|  Ok(PlayCard(cr.to_owned()))),
                    Some(s) => match self.situation.hand().cards_in_suit(s).iter().choose(&mut rng){
                        None => self.situation.cards_hand().iter().choose(&mut rng).map_or(Err(BridgeCoreErrorStd::Hand(EmptyHand)), |c| Ok(PlayCard(c.to_owned()))),
                        Some(c) => Ok(PlayCard(c.to_owned()))
                    }
                }

            },
            Some(dummy) if  dummy == self.situation.side().partner() => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_dummy().iter().choose(&mut rng)
                        .map_or(Err(BridgeCoreErrorStd::Hand(EmptyHand)), |cr|  Ok(PlayCard(cr.to_owned()))),
                    Some(s) => match self.situation.dummy_hand().cards_in_suit(s).iter().choose(&mut rng){
                        None => self.situation.cards_dummy().iter().choose(&mut rng).map_or(Err(BridgeCoreErrorStd::Hand(EmptyHand)), |c| Ok(PlayCard(c.to_owned()))),
                        Some(c) => Ok(PlayCard(c.to_owned()))
                    }
                }

            }
            Some(bad) => Err(ViolatedOrder(Mismatch{expected: bad, found: self.situation.side()}).into())
        }
    }
}

impl<Comm> AwareAgent<Situation> for DeclarerBot<Comm>
where Comm: CommunicationEnd<ClientDealMessage, ServerDealMessage, BridgeErrorStd>{
    fn env(&self) -> &Situation {
        &self.situation
    }

    fn env_mut(&mut self) -> &mut Situation {
        &mut self.situation
    }

    fn set_dummy_hand(&mut self, dummy_hand: BridgeHand) {
        self.env_mut().set_dummy(dummy_hand)
    }
}

impl<Comm> CommunicatingAgent<ServerDealMessage, ClientDealMessage,  DealAction, BridgeErrorStd> for DeclarerBot<Comm>
    where Comm: CommunicationEnd<ClientDealMessage, ServerDealMessage, BridgeErrorStd> {
    fn send(&mut self, message: ClientDealMessage) -> Result<(), BridgeErrorStd> {
        self.comm.send(message)
    }

    fn recv(&mut self) -> Result<ServerDealMessage, BridgeErrorStd> {
        self.comm.recv()
    }
}





