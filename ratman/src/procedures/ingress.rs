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
        fs::{File, OpenOptions},
        select,
        sync::{
            broadcast::{Receiver as BcastReceiver, Sender as BcastSender},
            mpsc::Receiver,
        },
        task::spawn,
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
                spawn(async move {
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
    let inner_manifest = manifest.manifest.maybe_inner()?;
    debug!("Attempt to reassemble message stream for manifest {inner_manifest:?}");

    // fixme: this won't work on non-linux?
    let (read_cap, letterhead) = match inner_manifest {
        ManifestFrame::V1(ref v1) => (
            <ManifestFrameV1 as Into<Result<ReadCapability>>>::into(v1.clone())?,
            v1.letterhead.clone(),
        ),
    };

    loop {
        let null_file = OpenOptions::new()
            .create(false)
            .write(true)
            .open("/dev/null")
            .await?;
        let mut compat_null = null_file.compat();

        // check that we have all the bits we need to decode a message stream.  If
        // we do we notify the API frontend
        match async_eris::decode(&mut compat_null, &read_cap, &ctx.journal.blocks).await {
            Ok(()) => break,
            Err(e) => {
                let tw = tripwire.clone();
                debug!("Couldn't re-assemble stream ({e}); wait for block notifier");
                drop(compat_null);
                select! {
                    biased;
                    _ = tw => break,
                    _ = block_notify.recv() => continue
                }
            }
        }
    }

    debug!("Passed re-assembly check!");

    match ctx
        .subs
        .recipients
        .lock()
        .await
        .get(&manifest.recipient)
        .ok_or(RatmanError::Nonfatal(NonfatalError::NoStream))
    {
        Ok(sub_id) => {
            // Notify all listening subscription streams
            if let Ok(bcast_tx) = ctx.get_active_listener(Some(*sub_id), letterhead.to).await {
                debug!("Notify subscription {sub_id}");
                bcast_tx.send((letterhead.clone(), read_cap)).unwrap();
            }

            Ok(())
        }
        Err(e) => {
            // Then notify all active sync listeners, if they exist
            if let Ok(bcast_tx) = ctx.get_active_listener(None, letterhead.to).await {
                debug!("Notify sync message receiver");
                bcast_tx.send((letterhead, read_cap)).unwrap();
                Ok(())
            } else {
                Err(e)
            }
        }
    }
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
