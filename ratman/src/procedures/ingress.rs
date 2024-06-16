// SPDX-FileCopyrightText: 2019-2024 Katharina Fey <kookie@spacekookie.de>
//
// SPDX-License-Identifier: AGPL-3.0-or-later WITH LicenseRef-AppStore

use crate::context::RatmanContext;
use async_eris::ReadCapability;
use libratman::{
    api::socket_v2::RawSocketHandle,
    frame::{
        carrier::{ManifestFrame, ManifestFrameV1},
        micro::MicroframeHeader,
    },
    tokio::{
        fs::File,
        select,
        sync::{
            broadcast::{Receiver as BcastReceiver, Sender as BcastSender},
            mpsc::Receiver,
        },
        task::spawn_local,
    },
    tokio_util::compat::TokioAsyncReadCompatExt,
    types::{AddrAuth, Ident32, LetterheadV1},
    NonfatalError, RatmanError, Result,
};
use std::sync::Arc;
use tripwire::Tripwire;

/// Notify the ingress system of a new manifest in the journal
pub(crate) struct MessageNotifier(pub Ident32);

/// Notify all stream assemblers that there's new blocks available
///
/// This is a technical limitation since we can't build an association between a
/// newly created block and a manifest that that block belongs to.  Only the
/// manifest will know whether it has all the blocks needed to reassemble a
/// message.  So on every new block we notify all assemblers currently waiting
/// for something to happen, and then fail again if the block was not part of
/// the stream they're reassembling.
#[derive(Clone)]
pub(crate) struct BlockNotifier;

/// Listen to manifest events, indicating that we should start assembling a full
/// message stream.  This spawns a task that will retry blocks that haven't made
/// it yet
pub async fn exec_ingress_system(
    ctx: Arc<RatmanContext>,
    mut rx: Receiver<MessageNotifier>,
    block_notifier: BcastSender<BlockNotifier>,
) {
    loop {
        let tripwire = ctx.tripwire.clone();
        let block_notifier = block_notifier.clone();
        select! {
            biased;
            _ = tripwire => break,
            manifest_notifier = rx.recv() => {
                if manifest_notifier.is_none() {
                    break;
                }

                let ctx = Arc::clone(&ctx);
                let tripwire = ctx.tripwire.clone();
                spawn_local(async move {
                    if let Err(e) = reassemble_message_stream(ctx, manifest_notifier.unwrap(), tripwire, block_notifier.subscribe()).await {
                        error!("message stream stuck: {e}");
                        return;
                    }
                });
            }
        }
    }

    info!("Ingress system shut down");
}

/// Decode a single message from a manifest/ read_capability
///
/// Spawn a task that calls this function again if it failed.
async fn reassemble_message_stream(
    ctx: Arc<RatmanContext>,
    manifest: MessageNotifier,
    tripwire: Tripwire,
    mut block_notify: BcastReceiver<BlockNotifier>,
) -> Result<()> {
    let manifest = ctx.journal.manifests.get(&manifest.0.to_string())?.unwrap();

    // fixme: this won't work on non-linux?
    let null_file = File::open("/dev/null").await?;
    let (read_cap, letterhead) = match manifest.manifest.maybe_inner()? {
        ManifestFrame::V1(ref v1) => (
            <ManifestFrameV1 as Into<Result<ReadCapability>>>::into(v1.clone())?,
            v1.letterhead.clone(),
        ),
    };

    // check that we have all the bits we need to decode a message stream.  If
    // we do we notify the API frontend
    if async_eris::decode(&mut null_file.compat(), &read_cap, &ctx.journal.blocks)
        .await
        .is_ok()
    {
        debug!("Assembled full block: {}", read_cap.root_reference);
    }
    // If we weren't able to decode the full stream we wait for a block notifier
    // event and then try again.
    else {
        loop {
            let tw = tripwire.clone();
            let null_file = File::open("/dev/null").await?;
            select! {
                biased;
                _ = tw => break,
                _ = block_notify.recv() => {
                    if async_eris::decode(&mut null_file.compat(), &read_cap, &ctx.journal.blocks)
                        .await
                        .is_ok()
                    {
                        debug!("Completed new block stream!");
                        break;
                    }
                }
            }
        }
    }

    let sub_id = *ctx
        .subs
        .recipients
        .lock()
        .await
        .get(&manifest.recipient)
        .ok_or(RatmanError::Nonfatal(NonfatalError::NoStream))?;

    // Notify all listening subscription streams
    if let Ok(bcast_tx) = ctx.get_active_listener(Some(sub_id), letterhead.to).await {
        bcast_tx.send((letterhead.clone(), read_cap)).unwrap();
    }

    // Then notify all active sync listeners, if they exist
    if let Ok(bcast_tx) = ctx.get_active_listener(None, letterhead.to).await {
        bcast_tx.send((letterhead, read_cap)).unwrap();
    }

    Ok(())
}

pub async fn handle_subscription_socket(
    ctx: Arc<RatmanContext>,
    mut rx: BcastReceiver<(LetterheadV1, ReadCapability)>,
    mut client_socket: RawSocketHandle,
    auth: AddrAuth,
    sub_id: Ident32,
) {
    loop {
        let tw = ctx.tripwire.clone();

        let item = select! {
            biased;
            _ = tw => break,
            item = rx.recv() => item,
        };

        match item {
            Err(_) => break,
            Ok((letterhead, read_cap)) => {
                use libratman::frame::micro::client_modes as cm;

                if let Err(e) = client_socket
                    .write_microframe(
                        MicroframeHeader {
                            modes: cm::make(cm::SUB, cm::ONE),
                            auth: Some(auth),
                            payload_size: 0,
                        },
                        letterhead.clone(),
                    )
                    .await
                {
                    error!("failed to send stream letterhead: {e}");
                    if let Err(e) = ctx.subs.missed_item(letterhead, read_cap).await {
                        error!("failed to persist missed item; client will miss this one: {e}");
                    }
                    break;
                }

                let mut compat_socket = client_socket.to_compat();

                // Stream the block stream to the client
                if let Err(e) =
                    async_eris::decode(&mut compat_socket, &read_cap, &ctx.journal.blocks)
                        .await
                        .map_err(|e| RatmanError::Block(libratman::BlockError::Eris(e)))
                {
                    error!("subscription stream has died: {e}");
                    if let Err(e) = ctx.subs.missed_item(letterhead, read_cap).await {
                        error!("failed to persist missed item; client will miss this one: {e}");
                    }
                    break;
                }

                client_socket.from_compat(compat_socket);
            }
        }
    }

    info!("Subscription socket {sub_id} terminated");
}
