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
};
use libratman::{
    api::types::ServerPing, frame::micro::MicroframeHeader, rt::new_async_thread, types::Ident32,
    ClientError, Result,
};
use libratman::{
    tokio::{
        io::ErrorKind as TokioIoErrorKind,
        net::{TcpListener, TcpStream},
        task::{spawn_local, yield_now},
    },
    RatmanError,
};
use std::{collections::BTreeMap, future::IntoFuture, net::SocketAddr, sync::Arc};

/// Start a new thread to run the client API socket
pub async fn start_api_thread(
    context: Arc<RatmanContext>,
    addr: SocketAddr,
    // config: &ConfigTree,
) -> Result<()> {
    new_async_thread("ratmand-api-acceptor", 1024, async move {
        let l = TcpListener::bind(addr).await?;

        while let Ok((stream, client_addr)) = l.accept().await {
            let jh = spawn_local(run_client_handler(Arc::clone(&context), stream));
            spawn_local(async move {
                let res = jh
                    .into_future()
                    .await
                    .expect("failed to join `run_client_handler` future");
                match res {
                    Ok(()) => info!("Client {client_addr:?} has disconnected gracefully!"),
                    Err(e) => error!("error occured while handling client connection: {e}"),
                }
            });
        }

        Ok(())
    });
    Ok(())
}

pub async fn run_client_handler(ctx: Arc<RatmanContext>, stream: TcpStream) -> Result<()> {
    let mut raw_socket = handshake(stream).await?;
    let mut active_auth = BTreeMap::new();
    let client_id = Ident32::random();

    // Add a new client entry for this session
    ctx.clients
        .lock_inner()
        .await
        .insert(client_id, Default::default());

    loop {
        match single_session_exchange(&ctx, client_id, &mut active_auth, &mut raw_socket).await {
            Ok(SessionResult::Next) => yield_now().await,
            Ok(SessionResult::Drop) => break,
            Err(RatmanError::TokioIo(io_err))
                if io_err.kind() == TokioIoErrorKind::UnexpectedEof =>
            {
                debug!("Unexpected end of file but we were probably expecting this");
            }
            Err(RatmanError::ClientApi(client_err)) => {
                debug!("Client {client_id} invoked an operation with invalid authentication");
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

    info!("Shutting down client socket {client_id}!");
    ctx.clients.lock_inner().await.remove(&client_id);

    Ok(())
}
