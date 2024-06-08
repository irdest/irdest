// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod client;

use crate::{config::ConfigTree, context::RatmanContext};
use libratman::{
    api::{socket_v2::RawSocketHandle, version_str, versions_compatible},
    rt::{new_async_thread, writer::AsyncWriter},
    tokio::{
        net::{unix::SocketAddr, TcpStream},
        sync::mpsc::{channel, Sender},
    },
    types::{Address, Id},
    ClientError, Result,
};
use std::sync::Arc;

pub(crate) struct ConnectionManager {}

/// Start a new thread to run the client API socket
pub fn start_message_acceptor(
    context: Arc<RatmanContext>,
    addr: SocketAddr,
    config: &ConfigTree,
) -> Result<()> {
    Ok(())
}

/// Initiate a new client connection
async fn handshake(context: Arc<RatmanContext>, stream: TcpStream) -> Result<Sender<Id>> {
    // Create a notifier channel for Subscription updates
    let (cl_notify_t, cl_notify_r) = channel(8);

    // Wrap the TcpStream to bring its API into scope
    let mut raw_socket = RawSocketHandle::new(stream);

    // Read the client API version to determine whether we are compatible
    let client_version = [0; 2];
    raw_socket.read_into(&mut client_version).await?;
    let compatible = versions_compatible(libratman::api::VERSION, client_version);

    // Reject connection and disconnect
    if !compatible {
        raw_socket
            .write_buffer(libratman::api::VERSION.to_vec())
            .await?;

        return Err(ClientError::IncompatibleVersion(format!(
            "self:{},client:{}",
            version_str(&libratman::api::VERSION),
            version_str(&client_version)
        ))
        .into());
    }

    // a) Server sends Ping
    // b) Client respond with Command or Pong::None

    // Wait for the Handshake reply from the client
    let hs_header = raw_socket.read_header().await?;

    
    
    Ok(cl_notify_t)
}

// async fn run_relay(context: Arc<RatmanContext>) {
//     loop {
//         let Message {
//             id,
//             sender,
//             recipient,
//             payload,
//             time,
//             signature,
//         } = context.core.next().await;
//         debug!("Receiving message for {:?}", recipient);
//         let recv = api::receive_default(Message::received(
//             id,
//             sender,
//             recipient.clone(),
//             payload,
//             format!("{:?}", time),
//             signature,
//         ));

//         match recipient {
//             ref recp @ ApiRecipient::Standard(_) => {
//                 let client_id = context
//                     .clients
//                     .get_client_for_address(&recp.scope().expect("empty recipient!"))
//                     .await;

//                 if client_id.is_none() {
//                     warn!("Received message for unknown address: {:?}!", recp.scope());
//                     continue;
//                 }

//                 let client_id = client_id.unwrap();

//                 // If the client wasn't online right now, the router
//                 // wouldn't have marked the message to be relayed, and
//                 // instead simply inserted it into the local journal.
//                 let mut online = context.clients.online.lock().await;
//                 if let Some(OnlineClient { ref mut io, .. }) = online.get_mut(&client_id) {
//                     info!("Forwarding message to online client!");
//                     if let Err(e) = parse::forward_recv(io.as_io(), recv).await {
//                         error!("Failed to forward received message: {}", e);
//                     }
//                 }
//             }
//             ApiRecipient::Flood(_) => {
//                 // TODO: how to determine whether a client has
//                 // "missed" a flood message.  Do we re-play flood
//                 // messages at all?  It could get quite big.
//                 let mut online = context.clients.online.lock().await;
//                 for (_, OnlineClient { ref mut io, .. }) in online.iter_mut() {
//                     if let Err(e) = parse::forward_recv(io.as_io(), recv.clone()).await {
//                         error!("Failed to forward received message: {}", e);
//                     }
//                 }
//             }
//         }
//     }
// }

// /// Listen for new connections on a socket address
// async fn listen_for_connections(
//     listen: &mut Incoming<'_>,
//     context: &Arc<RatmanContext>,
// ) -> Result<Option<(Address, Io)>> {
//     while let Some(stream) = listen.next().await {
//         let stream = stream?;
//         let mut io = Io::Tcp(stream);

//         let (id, _) = match parse::handle_auth(&mut io, &context).await {
//             Ok(Some(pair)) => {
//                 debug!("Successfully authenticated: {:?}", pair.0);
//                 pair
//             }

//             // An anonymous client doesn't need an entry in the
//             // lookup table because no message will ever be
//             // addressed to it
//             Ok(None) => return Ok(Some((Address::random(), io))),

//             Err(e) => {
//                 error!("Encountered error during auth: {}", e);
//                 break;
//             }
//         };

//         return Ok(Some((id, io)));
//     }

//     Ok(None)
// }

// /// Run the API receiver endpoint
// pub async fn run(context: Arc<RatmanContext>, addr: SocketAddr) -> Result<()> {
//     info!("Listening for API connections on socket {:?}", addr);
//     let listener = TcpListener::bind(addr).await?;
//     let mut incoming = listener.incoming();

//     let relay = task::spawn(run_relay(Arc::clone(&context)));

//     while let Ok(io) = listen_for_connections(&mut incoming, &context).await {
//         let (_self, io) = match io {
//             Some(io) => io,
//             // Broken connections get dropped
//             None => continue,
//         };

//         info!("Established new client connection");
//         task::spawn(parse::parse_stream(Arc::clone(&context), _self, io.clone()));
//     }

//     relay.cancel().await;
//     Ok(())
// }
