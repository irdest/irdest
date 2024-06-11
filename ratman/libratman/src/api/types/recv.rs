// pub struct RecvOne {}
// pub struct RecvMany {}
// pub struct RecvFetch {}

use crate::{
    frame::FrameGenerator,
    types::{Ident32, Recipient},
    Result,
};

pub struct SubsCreate {
    pub recipient: Recipient,
}

impl FrameGenerator for SubsCreate {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        Some(self.recipient).generate(buf)?;
        Ok(())
    }
}

pub struct SubsDelete {
    pub sub_id: Ident32,
}

impl FrameGenerator for SubsDelete {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        Some(self.sub_id).generate(buf)?;
        Ok(())
    }
}

pub struct SubsResponse {
    pub sub_id: Ident32,
}

impl FrameGenerator for SubsResponse {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        Some(self.sub_id).generate(buf)?;
        Ok(())
    }
}
