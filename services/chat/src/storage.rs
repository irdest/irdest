//! Irdest storage integration

use crate::{error::Result, types::Room};
use irdest_sdk::{
    users::{UserAuth, UserProfile},
    Identity, IrdestSdk,
};
use std::sync::Arc;

pub(crate) struct ServiceStore {
    i: Arc<IrdestSdk>,
}

impl ServiceStore {
    pub(crate) fn new(i: &Arc<IrdestSdk>) -> Self {
        Self { i: Arc::clone(&i) }
    }

    pub(crate) async fn generate_name(&self, p: &Vec<Identity>) -> String {
        let mut names = vec![];
        for id in p.iter() {
            let profile = self.i.users().get(*id).await;

            names.push(match profile {
                Ok(UserProfile {
                    handle: Some(handle),
                    ..
                }) => handle.clone(),
                _ => "Unknown".to_string(), // TODO: generate a random
                                            // name based on the ID!
            });
        }

        // Then concat them together
        names.join(" & ")
    }

    pub(crate) async fn create_room(
        &self,
        auth: UserAuth,
        p: Vec<Identity>,
        name: String,
    ) -> Result<Room> {
        todo!()
    }

    pub(crate) fn get_rooms(&self, auth: UserAuth) -> Result<Vec<Room>> {
        todo!()
    }
}
