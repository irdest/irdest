use crate::{
    frame::{
        parse::{take_id, take_u32, take_u64},
        FrameGenerator, FrameParser,
    },
    types::Ident32,
    Result,
};

/// Describe a neighbouring router node
pub struct RouterMeta {
    pub key_id: Ident32,
    pub available_buffer: u64,
    pub known_peers: u32,
}

impl FrameParser for RouterMeta {
    type Output = RouterMeta;

    fn parse(input: &[u8]) -> nom::IResult<&[u8], Self::Output> {
        let (input, key_id) = take_id(input)?;
        let (input, available_buffer) = take_u64(input)?;
        let (input, known_peers) = take_u32(input)?;

        Ok((
            input,
            RouterMeta {
                key_id,
                available_buffer,
                known_peers,
            },
        ))
    }
}

impl FrameGenerator for RouterMeta {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.key_id.generate(buf)?;
        self.available_buffer.generate(buf)?;
        self.known_peers.generate(buf)?;
        Ok(())
    }
}
