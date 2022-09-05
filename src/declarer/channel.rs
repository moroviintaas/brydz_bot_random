
use std::sync::mpsc::{Receiver, Sender};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::thread_rng;
use bridge_core::deal::{DealMaintainer};
use bridge_core::error::{BridgeError, BridgeErrorStd, DealError,  FlowError, Mismatch, TrickError};
use bridge_core::error::FlowError::{DifferentSideExpected, PlayAfterEnd, ServerDead};
use bridge_core::error::HandError::EmptyHand;
use bridge_core::karty::cards::CardStd;
use bridge_core::player::side::Side;
use bridge_core::player::situation::Situation;
use bridge_core::protocol::{ClientMessage, DealNotify, ServerMessage};
use bridge_core::protocol::DealAction::{NotMyTurn, PlayCard};
use crate::Bot;
use log::{ info, debug, error};

pub struct DeclarerOverChannel{
    sender: Sender<ClientMessage>,
    receiver: Receiver<ServerMessage>,
    situation: Situation,
    pending_card: bool,
}

impl DeclarerOverChannel{
    pub fn new(sender: Sender<ClientMessage>, receiver: Receiver<ServerMessage>, situation: Situation) -> Self{
        Self{sender, receiver, situation, pending_card: false}
    }
    fn select_card_from_hand(&mut self, rng: &mut ThreadRng) -> Result<CardStd, BridgeErrorStd> {
        match self.situation.current_side() {
            Some(s) if s == self.situation.side() => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_hand().iter().choose(rng)
                        .map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |cr|  Ok(cr.to_owned())),
                    Some(s) => match self.situation.hand().cards_in_suit(s).iter().choose(rng){
                        None => self.situation.cards_hand().iter().choose(rng).map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |c| Ok(c.to_owned())),
                        Some(c) => Ok(c.to_owned())
                    }
                }
            }
            Some(sheduled) => {
                Err(BridgeError::Trick(TrickError::ViolatedOrder(Mismatch { expected: sheduled, found: self.situation.side() })))
            }
            None => Err(BridgeError::Flow(PlayAfterEnd(self.side())))
        }

    }
    fn select_card_from_dummy(&mut self, rng: &mut ThreadRng) -> Result<CardStd, BridgeErrorStd> {
        match self.situation.current_side() {
            Some(s) if s == self.situation.side().partner() => {
                match self.situation.deal().current_trick().called_suit() {
                    None => self.situation.cards_dummy().iter().choose(rng)
                        .map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |cr|  Ok(cr.to_owned())),
                    Some(s) => match self.situation.dummy_hand().cards_in_suit(s).iter().choose(rng){
                        None => self.situation.cards_dummy().iter().choose(rng).map_or(Err(BridgeErrorStd::Hand(EmptyHand)), |c| Ok(c.to_owned())),
                        Some(c) => Ok(c.to_owned())
                    }
                }
            }
            Some(sheduled) => {
                Err(BridgeError::Trick(TrickError::ViolatedOrder(Mismatch { expected: sheduled, found: self.situation.side().partner() })))
            }
            None => Err(BridgeError::Flow(PlayAfterEnd(self.partner_side())))
        }

    }
    /*
    fn make_move(&mut self, rng: &mut ThreadRng) -> Result<(), BridgeErrorStd>{
        let c =  match self.current_side(){
            Some(s) if s == self.side() => self.select_card_from_hand(rng),
            Some(s) if s == self.partner_side() => self.select_card_from_dummy(rng),
            Some(other) => {
                self.sender.send(ClientMessage::Dealing(NotMyTurn))?;
                self.sender.send(ClientMessage::Quit)?;
                return Err(BridgeError::Trick(TrickError::ViolatedOrder(Mismatch{expected: other, found: other})));

            }
            None => {
                return Err(BridgeError::Flow(PlayAfterEnd(self.side())));
            }
        };
        match c {
            Ok(card) => {
                self.sender.send(ClientMessage::Dealing(PlayCard(self.side(), card)))?;
                self.pending_card = true;
                Ok(())

            }
            Err(e) => {
                self.sender.send(ClientMessage::Error(e.clone()))?;
                self.sender.send(ClientMessage::Quit)?;
                Err(e)
            }
        }
    }*/
    fn make_move(&mut self, rng: &mut ThreadRng) -> Result<(), BridgeErrorStd>{
        let c =  match self.current_side(){
            Some(s) if s == self.side() => self.select_card_from_hand(rng),
            Some(s) if s == self.partner_side() => self.select_card_from_dummy(rng),
            Some(other) => {
                self.sender.send(ClientMessage::Dealing(NotMyTurn))?;
                self.sender.send(ClientMessage::Quit)?;
                return Err(BridgeError::Trick(TrickError::ViolatedOrder(Mismatch{expected: other, found: other})));

            }
            None => {
                return Err(BridgeError::Flow(PlayAfterEnd(self.side())));
            }
        };
        match c {
            Ok(card) => {
                self.sender.send(ClientMessage::Dealing(PlayCard( card)))?;
                self.pending_card = true;
                Ok(())

            }
            Err(e) => {
                self.sender.send(ClientMessage::Error(e.clone()))?;
                self.sender.send(ClientMessage::Quit)?;
                Err(e)
            }
        }
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

impl Bot for DeclarerOverChannel{
    fn run(&mut self) -> Result<(), BridgeErrorStd> {
        debug!("Declarer ({:?}) hand: {:#}", self.side(), self.situation.hand());
        let mut rng = thread_rng();
        self.sender.send(ClientMessage::Ready)?;
        //let mut last_played_card: Option<CardStd> = None;
        loop{
            match self.receiver.recv(){
                Ok(message) => match message{
                    ServerMessage::Deal(notify) => match notify{
                        DealNotify::CardPlayed(side, card) => {
                            debug!("{:?} received info that player {:?}, played {:#}.", self.side(), side, card);
                            if let Err(e) = self.situation.mark_card_used(side, card){
                                error!("{:?} encountered error: {:?}", self.side(), e.clone());
                                self.sender.send(ClientMessage::Error(e.into()))?;

                            }

                            //panics when out of order card played
                            //todo log and ignore maybe
                        }
                        DealNotify::TrickClosed(_) => {}//trick should be automatically closed with last card in it,
                        //so I can ignore it for now
                        DealNotify::YourMove => {
                            debug!("{:?} declarer received signal to move.", self.side());
                            debug!("Declarer ({:?}) hand: {:#}", self.side(), self.situation.hand());
                            debug!("Declarer ({:?}) dummy_hand: {:#}", self.side(), self.situation.dummy_hand());
                            self.make_move(&mut rng)?;

                        }
                        DealNotify::CardAccepted(_card) => {
                            match self.current_side(){
                                Some(s) if s == self.side() || s == self.partner_side() => {}/*self.situation.mark_card_used(s, card)?*/,
                                Some(other) =>{
                                    error!("{:?} declarer different side expected: {:?}.", self.side(), other);
                                    return Err(BridgeError::Flow(DifferentSideExpected(other)));
                                }
                                None => {/*last card placed */  }
                            };
                            //self.situation.mark_card_used(self.side(), card)?;
                        }
                        DealNotify::CardDeclined(card) => {
                            self.sender.send(ClientMessage::Quit)?;
                            return Err(BridgeError::Deal(DealError::DuplicateCard(card)))
                        }
                        DealNotify::DummyPlacedHand(hand) => {
                            debug!("Declarer {:?} received message with dummy's hand: {:?}", &self.side(), &hand);
                            if self.situation.dummy_hand().cards().is_empty(){
                                self.situation.set_dummy(hand);
                            }
                            else{
                                self.sender.send(ClientMessage::Error(FlowError::ConfusingMessage.into()))?;
                                self.sender.send(ClientMessage::Quit)?
                            }

                        }
                        DealNotify::DealClosed => {
                            self.sender.send(ClientMessage::Quit)?;
                            return Ok(());
                        },
                    }
                    ServerMessage::Bidding(_) => {}
                    ServerMessage::PlayerLeft(_) => {}
                    ServerMessage::DealInfo(_) => {}
                    ServerMessage::BiddingInfo(_) => {}
                    ServerMessage::GameOver => {return Ok(())}
                    ServerMessage::Error(e) => {return Err(e)}
                    ServerMessage::ServerNotReady => {}
                    ServerMessage::ServerStopping => {
                        info!("Server is stopping. Exiting. {:?} signing out", self.side());
                        return Err(ServerDead.into());
                    },
                }
                Err(e) => return Err(e.into())
            }
        }

    }
}
