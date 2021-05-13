use crate::{Identity, IrdestSdk};

pub use irpc_sdk::{
    default_socket_path,
    error::{RpcError, RpcResult},
    io::{self, Message},
    RpcSocket, Service, SubSwitch, Subscription, ENCODING_JSON,
};
pub use std::{str, sync::Arc};

use ircore_types::rpc::{Capabilities, Reply, UserCapabilities, UserReply};
use ircore_types::users::{UserAuth, UserProfile, UserUpdate};

pub struct UserRpc<'ir> {
    pub(crate) rpc: &'ir IrdestSdk,
}

impl<'ir> UserRpc<'ir> {
    /// Enumerate all users available
    ///
    /// No information about sessions or existing login state is
    /// stored or accessible via this API.
    pub async fn list(&self) -> Vec<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::List))
            .await
        {
            Ok(Reply::Users(UserReply::List(list))) => list,
            _ => vec![],
        }
    }

    /// Enumerate remote stored users available
    pub async fn list_remote(&self) -> Vec<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::ListRemote))
            .await
        {
            Ok(Reply::Users(UserReply::List(list))) => list,
            _ => vec![],
        }
    }

    /// Check if a user ID and token combination is valid
    pub async fn is_authenticated(&self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::IsAuthenticated {
                auth,
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Create a new user and authenticated session
    ///
    /// The specified password `pw` is used to encrypt the user's
    /// private key and message stores and should be kept safe from
    /// potential attackers.
    ///
    /// It's mandatory to choose a password here, however it is
    /// possible for a frontend to choose a random sequence _for_ a
    /// user, instead of leaving files completely unencrypted. In this
    /// case, there's no real security, but a drive-by will still only
    /// grab encrypted files.
    pub async fn create(&self, pw: &str) -> RpcResult<UserAuth> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Create {
                pw: pw.into(),
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Auth(auth))) => Ok(auth),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Delete a local user from the auth store
    ///
    /// This function requires a valid login for the user that's being
    /// deleted.  This does not delete any data associated with this
    /// user, or messages from the node (or other device nodes).
    pub async fn delete(&self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Delete { auth }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Change the passphrase for an authenticated user
    pub async fn change_pw(&self, auth: UserAuth, new_pw: &str) -> RpcResult<()> {
        let new_pw = new_pw.to_string();
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::ChangePw {
                auth,
                new_pw,
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Create a new session login for a local User
    pub async fn login<S: Into<String>>(&self, id: Identity, pw: S) -> RpcResult<UserAuth> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Login {
                id,
                pw: pw.into(),
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Auth(auth))) => Ok(auth),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Drop the current session Token, invalidating it
    pub async fn logout(&self, auth: UserAuth) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Logout { auth }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Fetch the `UserProfile` for a known identity, remote or local
    ///
    /// No athentication is required for this endpoint, seeing as only
    /// public information is exposed via the `UserProfile`
    /// abstraction anyway.
    pub async fn get(&self, id: Identity) -> RpcResult<UserProfile> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Get { id }))
            .await
        {
            Ok(Reply::Users(UserReply::Profile(profile))) => Ok(profile),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }

    /// Update a `UserProfile` with a `UserUpdate` diff type
    pub async fn update(&self, auth: UserAuth, update: UserUpdate) -> RpcResult<()> {
        match self
            .rpc
            .send(Capabilities::Users(UserCapabilities::Update {
                auth,
                update,
            }))
            .await
        {
            Ok(Reply::Users(UserReply::Ok)) => Ok(()),
            Err(e) => Err(e),
            _ => Err(RpcError::EncoderFault("Invalid reply payload!".into())),
        }
    }
}
