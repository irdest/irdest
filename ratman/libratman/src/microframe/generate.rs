use crate::{types::frames::generate::any_as_u8_slice, Result};

pub trait MicroFrameGenerator {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()>;
}

impl<T: Sized> MicroFrameGenerator for T {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        buf.extend_from_slice(unsafe { any_as_u8_slice(&self) });
        Ok(())
    }
}
