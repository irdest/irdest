use crate::{
    diff::{ItemDiff, SetDiff},
    rpc::ConvertError,
    types_capnp::{
        item_diff,
        map::{self, entry},
        set_diff, tag, tag_set,
        user_update::bio_line,
    },
};
use alexandria_tags::{Tag, TagSet};
use capnp::{data, struct_list, text, text_list};
use qrpc_sdk::parser::MsgReader;
use std::collections::{BTreeMap, BTreeSet};

type TagsetReader<'s> = MsgReader<'s, tag_set::Reader<'s>>;
type TagReader<'s> = MsgReader<'s, tag::Reader<'s>>;

pub(crate) fn tagset_from_capnp<'s>(r: TagsetReader<'s>) -> Result<TagSet, ConvertError> {
    let rpc: tag_set::Reader = r.get_root().unwrap();
    let list = rpc.get_tags().unwrap();

    Ok(list.iter().fold(TagSet::empty(), |mut set, tag| {
        let k = tag.get_key().unwrap(); // FIXME :)
        let v = tag.get_val().unwrap();

        set.insert(Tag::new(k, v.iter().cloned()));
        set
    }))
}

pub(crate) fn map_from_capnp<'s>(
    mapr: map::Reader<'s, text::Owned, text::Owned>,
) -> Result<BTreeMap<String, String>, ConvertError> {
    let entries = mapr.get_entries().unwrap();

    Ok(entries.iter().fold(BTreeMap::new(), |mut map, entry| {
        let key = entry.get_key().unwrap();
        let value = entry.get_value().unwrap();

        map.insert(key.into(), value.into());
        map
    }))
}

pub(crate) fn set_from_capnp(listr: text_list::Reader) -> Result<BTreeSet<String>, ConvertError> {
    Ok(listr.iter().fold(BTreeSet::new(), |mut vec, serv| {
        vec.insert(serv.unwrap().into());
        vec
    }))
}

pub(crate) fn pair_list_from_capnp<'s>(
    listr: struct_list::Reader<'s, bio_line::Owned>,
) -> Result<Vec<(String, String)>, ConvertError> {
    listr.iter().fold(Ok(vec![]), |vec, bioline| {
        match (vec, pair_from_capnp(bioline)) {
            (Ok(mut vec), Ok(b)) => {
                vec.push(b);
                Ok(vec)
            }
            (Ok(_), Err(e)) => Err(e),
            (Err(e), _) => Err(e),
        }
    })
}

pub(crate) fn pair_from_capnp(pair: bio_line::Reader) -> Result<(String, String), ConvertError> {
    Ok((
        pair.get_key().map(|s| s.into())?,
        pair.get_val().map(|s| s.into())?,
    ))
}

pub(crate) fn avi_diff_from_capnp(avidiffr: item_diff::Reader<data::Owned>) -> ItemDiff<Vec<u8>> {
    use item_diff::Which::*;
    match avidiffr.which() {
        Ok(Set(Ok(t))) => ItemDiff::Set(t.into()),
        Ok(Unset(_)) => ItemDiff::Unset,
        _ => ItemDiff::Ignore,
    }
}

pub(crate) fn text_diff_from_capnp(
    tdiff: Option<item_diff::Reader<text::Owned>>,
) -> ItemDiff<String> {
    use item_diff::Which::*;
    match tdiff {
        Some(t) => match t.which() {
            Ok(Set(Ok(t))) => ItemDiff::Set(t.into()),
            Ok(Unset(_)) => ItemDiff::Unset,
            _ => ItemDiff::Ignore,
        },
        _ => ItemDiff::Ignore,
    }
}

pub(crate) fn set_diff_from_capnp(sdiff: &set_diff::Reader<text::Owned>) -> SetDiff<String> {
    use set_diff::Which::*;
    match sdiff.which() {
        Ok(Add(Ok(t))) => SetDiff::Add(t.into()),
        Ok(Remove(Ok(t))) => SetDiff::Remove(t.into()),
        _ => SetDiff::Ignore,
    }
}

// pub(crate) fn item_diff_from_capnp<'s, T, Q>(idiffr: item_diff::Reader<'s, Q>) -> ItemDiff<T>
// where
//     for<'c> Q: Into<T> + capnp::traits::Owned<'c>,
//     T: From<<Q as capnp::traits::Owned<'s>>::Reader>,
// {
//     match idiffr.which() {
//         Ok(item_diff::Which::Set(Ok(t))) => ItemDiff::Set(t.into()),
//         Ok(item_diff::Which::Unset(_)) => ItemDiff::Unset,
//         _ => ItemDiff::Ignore,
//     }
// }

// struct Data(Vec<u8>);
// impl<'s> capnp::traits::Owned<'s> for Data {
//     type Reader = data::Reader<'s>;
//     type Builder = data::Builder<'s>;
// }

// impl TryFrom<TagsetReader> for BTreeMap<String, String> {
//     type Error = ConvertError;

//     fn try_from(rpc: TagsetReader) -> Result<Self, Self::Error> {
//         let rpc: tag_set::Reader = match rpc.get_root() {
//             Ok(u) => Ok(u),
//             Err(e) => Err(ConvertError::BaseDecodeError(e.to_string())),
//         }?;

//         // let id = Identity::from_string(&try_read(rpc.get_id(), "id")?.to_string());
//         // let token = try_read(rpc.get_token(), "token")?.into();

//         // Ok(UserAuth(id, token))

//         todo!()
//     }
// }
