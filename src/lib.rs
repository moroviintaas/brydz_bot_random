use brydz_framework::error::BridgeErrorStd;

pub mod declarer;
pub mod defender;
pub mod dummy;

pub trait Bot{

    fn run(&mut self) -> Result<(), BridgeErrorStd>;
}

