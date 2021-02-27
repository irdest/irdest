//! A set of API traits for libqaul

use crate::users::{UserAuth, UserProfile, UserUpdate};
use crate::{error::Result, Identity};

#[async_trait::async_trait]
pub trait UserApi {
    /// Enumerate all users available
    ///
    /// No information about sessions or existing login state is
    /// stored or accessible via this API.
    async fn list(&self) -> Vec<UserProfile>;

    /// Enumerate remote stored users available
    async fn list_remote(&self) -> Vec<UserProfile>;

    /// Check if a user ID and token combination is valid
    async fn is_authenticated(&self, user: UserAuth) -> Result<()>;

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
    async fn create(&self, pw: &str) -> Result<UserAuth>;

    /// Delete a local user from the auth store
    ///
    /// This function requires a valid login for the user that's being
    /// deleted.  This does not delete any data associated with this
    /// user, or messages from the node (or other device nodes).
    async fn delete(&self, user: UserAuth) -> Result<()>;

    /// Change the passphrase for an authenticated user
    fn change_pw(&self, user: UserAuth, newpw: &str) -> Result<()>;

    /// Create a new session login for a local User
    async fn login(&self, user: Identity, pw: &str) -> Result<UserAuth>;

    /// Drop the current session Token, invalidating it
    async fn logout(&self, user: UserAuth) -> Result<()>;

    /// Fetch the `UserProfile` for a known identity, remote or local
    ///
    /// No athentication is required for this endpoint, seeing as only
    /// public information is exposed via the `UserProfile`
    /// abstraction anyway.
    async fn get(&self, user: Identity) -> Result<UserProfile>;

    /// Update a `UserProfile` with a lambda, if authentication passes
    async fn update(&self, user: UserAuth, update: UserUpdate) -> Result<()>;
}
