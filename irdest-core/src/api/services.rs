//! Service interconnect interface

use crate::{
    error::Result,
    helpers::TagSet,
    services::{ServiceEvent, StoreKey},
    users::UserAuth,
    Irdest,
};
use serde::Serialize;

/// Manage service sessions and related metadata
///
/// Services are external micro-applications using irdest-core as a
/// basis to communicate on a distributed network.  For a service to
/// start using all irdest-core functions it must register itself via
/// this API.  This will unlock internal storage and user event
/// subscription support.
///
/// Some applications will be very security critical, and so, there
/// needs to be a way to store data in a safe way for future sessions,
/// without offering metadata sidechannels from captured devices.
/// This API is also a solution to this problem.
///
/// In irdest-core, all data is stored to disk encrypted, meaning that
/// conversations, keys and logs are safe from inspection.  To allow
/// services to hook into the same storage mechanism for their own
/// metadata, this API provides a view into a per-user, per-service
/// key-value store.  This way your service doesn't have to
/// re-implemented secure disk storage, or rely on easier non-secure
/// storage.
pub struct Services<'chain> {
    pub(crate) q: &'chain Irdest,
}

impl<'ird> Services<'ird> {
    /// Check if "god mode" is supported by this instance
    pub fn god_mode(&self) -> bool {
        true // TODO: make configurable
    }

    /// Add an external service to the irdest service registry
    ///
    /// Registering a service means that future `Message` listeners
    /// can be allocated for this service, as well as enabling polling.
    ///
    /// Names of services need to be unique, so it's advised to
    /// namespace them on some other key, for example the application
    /// package name (such as `com.example.myapp`)
    ///
    /// Check the [developer manual]() for a list of service addresses
    /// used by the irdest project!
    pub async fn register<S: Into<String>, F: 'static>(&self, name: S, cb: F) -> Result<()>
    where
        F: Fn(ServiceEvent) + Send + Sync,
    {
        self.q.services.register(name.into(), cb).await
    }

    /// Remove an external service from the irdest service registry
    ///
    /// Calling this function will disable the ability to poll for
    /// messages, as well as deleting all already registered message
    /// listeners existing for this service.
    ///
    /// Will return `Error::NoService` if no such service name could
    /// be found.
    pub async fn unregister<S: Into<String>>(&self, name: S) -> Result<()> {
        self.q.services.unregister(name.into()).await
    }

    /// Store some service data in irdest-core
    ///
    /// A service can store metadata in the same encrypted database
    /// that irdest-core uses.  Each service has access to a single
    /// map ([`MetadataMap`]()), which maps `StoreKey`s to arbitrary
    /// byte arrays.  A [`StoreKey`] is a 2-String tuple.  A service
    /// MAY use this structure to create namespaces inside the map.
    pub async fn insert<S, K>(
        &self,
        user: UserAuth,
        service: S,
        key: K,
        value: Vec<u8>,
    ) -> Result<()>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        // Verify this request is valid
        let serv = service.into();
        let (id, _) = self.q.auth.trusted(user)?;
        self.q.services.check(&serv).await?;

        // Then call insert on the store
        self.q
            .services
            .store()
            .insert(id, serv, key.into(), value)
            .await
    }

    /// Delete a particular key from the service metadata store
    ///
    /// This function Will only return an error for access permission
    /// failures, not if the data key didn't previously exist.
    pub async fn delete<S, K>(&self, user: UserAuth, service: S, key: K) -> Result<()>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        // Verify this request is valid
        let serv = service.into();
        let (id, _) = self.q.auth.trusted(user)?;
        self.q.services.check(&serv).await?;

        // Then call delete on the store
        self.q.services.store().delete(id, serv, key.into()).await;
        Ok(())
    }

    /// Make a query into the service metadata store
    ///
    /// You can only return one entry at a time, indexed by the
    /// StoreKey.  A store key is a 2-String tuple of namespace and
    /// key.
    pub async fn query<S, K>(&self, user: UserAuth, service: S, key: K) -> Result<Vec<u8>>
    where
        S: Into<String>,
        K: Into<StoreKey>,
    {
        // Verify this request is valid
        let serv = service.into();
        let (id, _) = self.q.auth.trusted(user)?;
        self.q.services.check(&serv).await?;

        // Then call query on the store
        self.q.services.store().query(id, serv, key.into()).await
    }
}
