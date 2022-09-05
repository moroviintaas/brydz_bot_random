
use bridge_core::error::{BridgeErrorStd};

pub mod declarer;
pub mod defender;
pub mod dummy;

pub trait Bot{

    fn run(&mut self) -> Result<(), BridgeErrorStd>;
}

