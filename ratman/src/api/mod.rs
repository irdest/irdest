// SPDX-FileCopyrightText: 2023 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod client;
mod connection;
mod io;
mod parse;

pub(self) use self::io::Io;
use crate::{api::client::OnlineClient, context::RatmanContext};
use async_std::{
    net::{Incoming, SocketAddr, TcpListener},
    stream::StreamExt,
    sync::Arc,
    task,
};
pub(crate) use client::BaseClient;
pub(crate) use connection::ConnectionManager;
use libratman::{
    types::{api, Address, Message, Recipient},
    Result,
};

async fn run_relay(context: Arc<RatmanContext>) {
    loop {
        let Message {
            id,
            sender,
            recipient,
            payload,
            time,
            signature,
        } = context.core.next().await;
        debug!("Receiving message for {:?}", recipient);
        let recv = api::receive_default(Message::received(
            id,
            sender,
            recipient.clone(),
            payload,
            format!("{:?}", time),
            signature,
        ));

        match recipient {
            ref recp @ Recipient::Standard(_) => {
                let client_id = context
                    .clients
                    .get_client_for_address(&recp.scope().expect("empty recipient!"))
                    .await;

                // If the client wasn't online right now, the router
                // wouldn't have marked the message to be relayed, and
                // instead simply inserted it into the local journal.
                let mut online = context.clients.online.lock().await;
                if let Some(OnlineClient { ref mut io, .. }) = online.get_mut(&client_id) {
                    info!("Forwarding message to online client!");
                    if let Err(e) = parse::forward_recv(io.as_io(), recv).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
            Recipient::Flood(_) => {
                // TODO: how to determine whether a client has
                // "missed" a flood message.  Do we re-play flood
                // messages at all?  It could get quite big.
                let mut online = context.clients.online.lock().await;
                for (_, OnlineClient { ref mut io, .. }) in online.iter_mut() {
                    if let Err(e) = parse::forward_recv(io.as_io(), recv.clone()).await {
                        error!("Failed to forward received message: {}", e);
                    }
                }
            }
        }
    }
}

/// Listen for new connections on a socket address
async fn listen_for_connections(
    listen: &mut Incoming<'_>,
    context: &Arc<RatmanContext>,
) -> Result<Option<(Address, Io)>> {
    while let Some(stream) = listen.next().await {
        let stream = stream?;
        let mut io = Io::Tcp(stream);

        let (id, _) = match parse::handle_auth(&mut io, &context).await {
            Ok(Some(pair)) => {
                debug!("Successfully authenticated: {:?}", pair.0);
                pair
            }

            // An anonymous client doesn't need an entry in the
            // lookup table because no message will ever be
            // addressed to it
            Ok(None) => return Ok(Some((Address::random(), io))),
            Err(e) => {
                error!("Encountered error during auth: {}", e);
                break;
            }
        };

        return Ok(Some((id, io)));
    }

    Ok(None)
}

/// Run the API receiver endpoint
pub async fn run(context: Arc<RatmanContext>, addr: SocketAddr) -> Result<()> {
    info!("Listening for API connections on socket {:?}", addr);
    let listener = TcpListener::bind(addr).await?;
    let mut incoming = listener.incoming();

    let relay = task::spawn(run_relay(Arc::clone(&context)));

    while let Ok(io) = listen_for_connections(&mut incoming, &context).await {
        let (_self, io) = match io {
            Some(io) => io,
            // FIXME: what are anonymous clients for again ?
            None => continue,
        };

        info!("Established new client connection");
        task::spawn(parse::parse_stream(Arc::clone(&context), _self, io.clone()));
    }

    relay.cancel().await;
    Ok(())
}
