use crate::{
    microframe::{client_modes::*, MicroframeHeader},
    types::frames::parse::{self, take_u16_slice},
    Result,
};
use async_std::{channel::Sender, net::TcpStream, sync::Mutex};

/// Api and state abstraction for a Ratman Api client
pub struct RatmanClient {}

struct RawSocketHandle(TcpStream, Sender<()>);

impl RawSocketHandle {

    /// 
    pub async fn startup(mut self) -> Result<()> {
        // We start by writing our versioned HELLO o/ message into the
        // stream.  We then WAIT for a client to answer with a
        // request.  After 10 seconds we give up.

        
        

        
        Ok(())
    }
}
