// SPDX-FileCopyrightText: 2023-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

mod clients;
pub(crate) use clients::ConnectionManager;

mod receiving;
mod sending;
mod session;

use crate::{
    api::session::{handshake, single_session_exchange, SessionResult},
    context::RatmanContext,
    procedures::SenderSystem,
};
use libratman::{
    api::types::ServerPing, frame::micro::MicroframeHeader, rt::new_async_thread, types::Ident32,
    ClientError, Result,
};
use libratman::{
    tokio::{
        io::ErrorKind as TokioIoErrorKind,
        net::{TcpListener, TcpStream},
        task::{spawn, yield_now},
    },
    RatmanError,
};
use std::{future::IntoFuture, net::SocketAddr, sync::Arc};

/// Start a new thread to run the client API socket
pub async fn start_api_thread(
    context: Arc<RatmanContext>,
    addr: SocketAddr,
    senders: Arc<SenderSystem>,
) -> Result<()> {
    new_async_thread("ratmand-api-acceptor", 1024, async move {
        info!("Listening to API socket on {addr}");
        let l = TcpListener::bind(addr).await?;

        while let Ok((stream, client_addr)) = l.accept().await {
            let client_id = Ident32::random();
            debug!("Accepted new api client {}", client_id.pretty_string());

            let jh = spawn(run_client_handler(
                Arc::clone(&context),
                Arc::clone(&senders),
                stream,
                client_id,
            ));

            debug!("Starting new api thread");
            let ctx = Arc::clone(&context);
            new_async_thread(
                format!("ratmand-api-{}", client_id.to_string().to_ascii_lowercase()),
                1024 * 16,
                async move {
                    debug!("Oiiii");
                    let res = jh
                        .into_future()
                        .await
                        .expect("failed to join `run_client_handler` future");

                    debug!("AWAWAWAWAWA");

                    // Remove the client here, no matter what the runner task does
                    ctx.clients.lock_inner().await.remove(&client_id);
                    match res {
                        Ok(()) => {
                            debug!("Client {client_addr:?} has disconnected gracefully!");
                            Ok(())
                        }
                        Err(e) => {
                            error!("error occured while handling client connection: {e}");
                            Err(e)
                        }
                    }
                },
            );
        }

        Ok(())
    });
    Ok(())
}

pub async fn run_client_handler(
    ctx: Arc<RatmanContext>,
    senders: Arc<SenderSystem>,
    stream: TcpStream,
    client_id: Ident32,
) -> Result<()> {
    let mut raw_socket = handshake(stream).await?;

    // Add a new client entry for this session
    ctx.clients
        .lock_inner()
        .await
        .insert(client_id, Default::default());

    loop {
        let auth_guard = ctx.clients.active_auth();
        match single_session_exchange(&ctx, client_id, &auth_guard, &mut raw_socket, &senders).await
        {
            Ok(SessionResult::Next) => {
                drop(auth_guard);
                yield_now().await;
            }
            Ok(SessionResult::Drop) => break,
            Err(RatmanError::TokioIo(io_err))
                if io_err.kind() == TokioIoErrorKind::UnexpectedEof =>
            {
                debug!("Unexpected end of file but we were probably expecting this");
            }
            Err(RatmanError::ClientApi(client_err)) => {
                debug!("Client {client_id} failed authentication: {client_err}");
                raw_socket
                    .write_microframe(
                        MicroframeHeader::intrinsic_noauth(),
                        ServerPing::Error(client_err),
                    )
                    .await?;
            }
            Err(e) => {
                // Terminate the session and send error payload to client
                error!("Fatal error occured in client session: {e}");
                raw_socket
                    .write_microframe(
                        MicroframeHeader::intrinsic_noauth(),
                        ServerPing::Error(ClientError::Internal(e.to_string())),
                    )
                    .await?;
                break;
            }
        }
    }
    Ok(())
}
