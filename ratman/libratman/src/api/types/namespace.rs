use crate::{
    frame::{
        parse::{take_address, take_id, take_u128},
        FrameGenerator, FrameParser,
    },
    types::{Address, Ident32},
    Result,
};
use nom::IResult;

/// Register a new namespace key and subscribe to it on the server side (for a
/// given application/ address pair)
pub struct NamespaceRegister {
    pub pubkey: Address,
    pub privkey: Ident32,
}

impl FrameGenerator for NamespaceRegister {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.pubkey.generate(buf)?;
        self.privkey.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for NamespaceRegister {
    type Output = NamespaceRegister;

    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, pubkey) = take_address(input)?;
        let (input, privkey) = take_id(input)?;

        Ok((input, NamespaceRegister { pubkey, privkey }))
    }
}

pub struct AnycastProbe {
    pub pubkey: Address,
    pub timeout_ms: u128,
}

impl FrameGenerator for AnycastProbe {
    fn generate(self, buf: &mut Vec<u8>) -> Result<()> {
        self.pubkey.generate(buf)?;
        self.timeout_ms.generate(buf)?;
        Ok(())
    }
}

impl FrameParser for AnycastProbe {
    type Output = AnycastProbe;
    fn parse(input: &[u8]) -> IResult<&[u8], Self::Output> {
        let (input, pubkey) = take_address(input)?;
        let (input, timeout_ms) = take_u128(input)?;

        Ok((input, AnycastProbe { pubkey, timeout_ms }))
    }
}
