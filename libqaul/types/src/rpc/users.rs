use crate::{
    diff::ItemDiff,
    rpc::{try_read, util, ConvertError},
    types_capnp::{user_auth, user_profile, user_update},
    users::{UserAuth, UserProfile, UserUpdate},
    Identity,
};
use capnp::{message::Builder, serialize_packed};
use qrpc_sdk::{error::RpcResult, io::WriteToBuf, parser::MsgReader};
use std::collections::BTreeSet;
use std::convert::TryFrom;

///////////////// UserAuth

type UserAuthReader = MsgReader<'static, user_auth::Reader<'static>>;

impl TryFrom<UserAuthReader> for UserAuth {
    type Error = ConvertError;

    fn try_from(rpc: UserAuthReader) -> Result<Self, Self::Error> {
        let rpc: user_auth::Reader = match rpc.get_root() {
            Ok(u) => Ok(u),
            Err(e) => Err(ConvertError::BaseDecodeError(e.to_string())),
        }?;

        let mut missing = vec![];

        let id = try_read(&mut missing, rpc.get_id(), "id")
            .map(|id| Identity::from_string(&id.to_string()));
        let token = try_read(&mut missing, rpc.get_token(), "token").map(|t| t.into());

        if !missing.is_empty() {
            return Err(ConvertError::MissingFields(missing));
        }

        Ok(UserAuth(id.unwrap(), token.unwrap()))
    }
}

impl WriteToBuf for UserAuth {
    fn to_vec(&self) -> RpcResult<Vec<u8>> {
        let mut msg = Builder::new_default();
        let mut root = msg.init_root::<user_auth::Builder>();
        root.set_id(&self.0.to_string());
        root.set_token(&self.1);

        let mut buffer = vec![];
        serialize_packed::write_message(&mut buffer, &msg)?;
        Ok(buffer)
    }
}

///////////////// UserProfile

type UserProfileReader<'s> = MsgReader<'s, user_profile::Reader<'s>>;

impl<'s> TryFrom<UserProfileReader<'s>> for UserProfile {
    type Error = ConvertError;

    fn try_from(r: UserProfileReader<'s>) -> Result<Self, Self::Error> {
        let rpc: user_profile::Reader = match r.get_root() {
            Ok(u) => Ok(u),
            Err(e) => Err(ConvertError::BaseDecodeError(e.to_string())),
        }?;

        let mut missing = vec![];

        let id = try_read(&mut missing, rpc.get_id(), "id")
            .map(|id| Identity::from_string(&id.to_string()));
        let handle = try_read(&mut missing, rpc.get_handle(), "handle");
        let display_name = try_read(&mut missing, rpc.get_display_name(), "display_name");
        let bio = try_read(&mut missing, rpc.get_bio(), "bio");
        let services = try_read(&mut missing, rpc.get_services(), "services");
        let avatar = try_read(&mut missing, rpc.get_avatar(), "avatar");

        if !missing.is_empty() {
            return Err(ConvertError::MissingFields(missing));
        }

        Ok(Self {
            id: id.unwrap().into(),
            handle: Some(handle.unwrap().into()),
            display_name: Some(display_name.unwrap().into()),
            bio: util::map_from_capnp(bio.unwrap())?,
            services: util::set_from_capnp(services.unwrap())?,
            avatar: Some(avatar.unwrap().into()),
        })
    }
}

///////////////// UserUpdate

type UserUpdateReader<'s> = MsgReader<'s, user_update::Reader<'s>>;

impl<'s> TryFrom<UserUpdateReader<'s>> for UserUpdate {
    type Error = ConvertError;

    fn try_from(r: UserUpdateReader<'s>) -> Result<UserUpdate, ConvertError> {
        let rpc: user_update::Reader = match r.get_root() {
            Ok(u) => Ok(u),
            Err(e) => Err(ConvertError::BaseDecodeError(e.to_string())),
        }?;

        let mut missing = vec![];

        let handle = util::text_diff_from_capnp(rpc.get_handle().ok());
        let display_name = util::text_diff_from_capnp(rpc.get_display_name().ok());

        let add_to_bio = match try_read(&mut missing, rpc.get_add_to_bio(), "add_to_bio") {
            Some(list) => util::pair_list_from_capnp(list)?,
            None => vec![],
        };

        let rm_from_bio = match try_read(&mut missing, rpc.get_rm_from_bio(), "rm_from_bio") {
            Some(list) => util::set_from_capnp(list)?,
            None => BTreeSet::new(),
        };

        let services = rpc
            .get_services()
            .map(|list| {
                list.iter().fold(BTreeSet::new(), |mut set, entry| {
                    let diff = util::set_diff_from_capnp(&entry);
                    set.insert(diff);
                    set
                })
            })
            .unwrap_or_else(|_| BTreeSet::new());

        // let add_service = match try_read(&mut missing, rpc.get_add_service(), "add_service") {
        //     Some(list) => util::set_from_capnp(list)?,
        //     None => BTreeSet::new(),
        // };
        // let rm_service = match try_read(&mut missing, rpc.get_rm_service(), "rm_service") {
        //     Some(list) => util::set_from_capnp(list)?,
        //     None => BTreeSet::new(),
        // };

        let avi_data = match try_read(&mut missing, rpc.get_avi_data(), "avi_data") {
            Some(d) => util::avi_diff_from_capnp(d),
            None => ItemDiff::Ignore,
        };

        Ok(UserUpdate {
            handle,
            display_name,
            add_to_bio,
            rm_from_bio,
            services,
            avi_data,
        })
    }
}
