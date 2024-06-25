use crate::storage::{subs::SubscriptionData, MetadataDb};
use async_eris::ReadCapability;
use libratman::{
    tokio::sync::{
        broadcast::{channel, Receiver, Sender},
        Mutex,
    },
    types::{Address, Ident32, LetterheadV1, Recipient},
    ClientError, RatmanError, Result,
};
use std::{collections::BTreeMap, sync::Arc};

type Locked<K, V> = Mutex<BTreeMap<K, V>>;

pub struct SubsManager {
    meta_db: Arc<MetadataDb>,
    pub(crate) recipients: Locked<Recipient, Ident32>,
    pub(crate) active_listeners: Locked<Ident32, Sender<(LetterheadV1, ReadCapability)>>,
}

impl SubsManager {
    pub fn new(meta_db: &Arc<MetadataDb>) -> Arc<Self> {
        let mut recipients = BTreeMap::new();

        meta_db
            .subscriptions
            .iter()
            .into_iter()
            .for_each(|(sub_id, sub_data)| {
                let id = Ident32::from_string(&sub_id);
                info!("Restore subscription {} from disk", id.pretty_string());
                recipients.insert(sub_data.recipient, id);
            });

        Arc::new(Self {
            meta_db: Arc::clone(meta_db),
            recipients: Locked::new(recipients),
            active_listeners: Locked::default(),
        })
    }

    async fn sub_listener(
        self: &Arc<Self>,
        sub_id: Ident32,
    ) -> Sender<(LetterheadV1, ReadCapability)> {
        self.active_listeners
            .lock()
            .await
            .entry(sub_id)
            .or_insert(channel(64).0)
            .clone()
    }

    pub fn available_subscriptions(self: &Arc<Self>, _recipient: Recipient) -> Vec<Ident32> {
        self.meta_db
            .subscriptions
            .iter()
            .into_iter()
            .map(|(sub_id, _)| Ident32::from_string(&sub_id))
            .collect()
    }

    pub async fn create_subscription(
        self: &Arc<Self>,
        addr: Address,
        recipient: Recipient,
    ) -> Result<(Ident32, Receiver<(LetterheadV1, ReadCapability)>)> {
        match self
            .meta_db
            .subscriptions
            .iter()
            .into_iter()
            .find(|(_, SubscriptionData { recipient: r, .. })| r == &recipient)
        {
            Some((sub_key, mut sub_val)) => {
                let sub_id = Ident32::from_string(&sub_key);

                // Update the existing subscription
                sub_val.listeners.insert(addr);

                debug!("Previous subscription found and updated: {sub_val:?}");
                self.meta_db
                    .subscriptions
                    .insert(sub_key.clone(), &sub_val)
                    .await?;

                // And then add a new active listeners stream
                let tx = self.sub_listener(sub_id).await;
                Ok((sub_id, tx.subscribe()))
            }
            None => {
                let sub_id = Ident32::random();
                debug!("Insert brand new subscription: {}", sub_id.pretty_string());
                self.meta_db
                    .subscriptions
                    .insert(
                        sub_id.to_string(),
                        &SubscriptionData {
                            recipient,
                            listeners: vec![addr].into_iter().collect(),
                            missed_items: Default::default(),
                        },
                    )
                    .await?;

                debug!(
                    "{:?}",
                    self.meta_db
                        .subscriptions
                        .iter()
                        .into_iter()
                        .map(|(k, v)| (k, v.recipient.inner_address()))
                        .collect::<Vec<(String, Address)>>()
                );

                // Update in-memory state for stream listener lookup
                self.recipients.lock().await.insert(recipient, sub_id);

                // Then insert and return a new listener stream
                let tx = self.sub_listener(sub_id).await;
                Ok((sub_id, tx.subscribe()))
            }
        }
    }

    pub async fn delete_subscription(
        self: &Arc<Self>,
        addr: Address,
        sub_id: Ident32,
    ) -> Result<()> {
        let mut sub = self
            .meta_db
            .subscriptions
            .get(&sub_id.to_string())
            .await?
            .ok_or(RatmanError::ClientApi(ClientError::NoSuchSubscription(
                sub_id,
            )))?;

        // Remove this address from the subscription and if the listener set is
        // empty afterwards we delete the whole subscription.  If anyone else is
        // still listening to it we keep it alive
        sub.listeners.remove(&addr);
        if sub.listeners.is_empty() {
            self.meta_db
                .subscriptions
                .remove(sub_id.to_string())
                .await?;
            self.recipients.lock().await.remove(&sub.recipient);
            self.active_listeners.lock().await.remove(&sub_id);
        } else {
            self.meta_db
                .subscriptions
                .insert(sub_id.to_string(), &sub)
                .await?;
            // If other listeners still exist for this subscription we don't
            // have to touch the listener set since we use a broadcast channel.
        }

        Ok(())
    }

    // This function only checks whether the subscription is valid and the
    // Address is indeed listening to this recipient.  If not, we return an
    // "NoAddress" error.
    pub async fn restore_subscription(
        self: &Arc<Self>,
        addr: Address,
        sub_id: Ident32,
    ) -> Result<Receiver<(LetterheadV1, ReadCapability)>> {
        if self
            .meta_db
            .subscriptions
            .get(&sub_id.to_string())
            .await?
            .ok_or(RatmanError::ClientApi(ClientError::NoSuchSubscription(
                sub_id,
            )))?
            .listeners
            .contains(&addr)
        {
            let tx = self.sub_listener(sub_id).await;

            Ok(tx.subscribe())
        } else {
            Err(RatmanError::ClientApi(ClientError::NoAddress))
        }
    }

    pub async fn missed_item(
        self: &Arc<Self>,
        letterhead: LetterheadV1,
        read_cap: ReadCapability,
    ) -> Result<()> {
        let sid = *self.recipients.lock().await.get(&letterhead.to).unwrap();
        let mut sentry = self
            .meta_db
            .subscriptions
            .get(&sid.to_string())
            .await?
            .unwrap();

        sentry
            .missed_items
            .entry(letterhead.to)
            .or_default()
            .push((letterhead, read_cap));

        self.meta_db
            .subscriptions
            .insert(sid.to_string(), &sentry)
            .await?;

        Ok(())
    }
}
