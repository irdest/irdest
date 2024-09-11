use crate::{frame::FrameGenerator, types::Ident32, Result};

/// Describe a neighbouring router node
pub struct RouterMeta {
    pub key_id: Ident32,
    pub available_buffer: Option<u64>,
    pub known_peers: u32,
}

impl FrameGenerator for RouterMeta {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.key_id.generate(buf)?;
        self.available_buffer.generate(buf)?;
        self.known_peers.generate(buf)?;
        Ok(())
    }
}
