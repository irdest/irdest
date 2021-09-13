//! Provides more convenient crypto wrappers
#![allow(unused)]

pub(crate) mod aes;
pub(crate) mod bs;
pub(crate) mod pkcry;

mod hidden;
pub(crate) use hidden::Hid;

use crate::{
    error::{Error, Result},
    utils::Id,
};
use async_std::sync::Arc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

/// An encrypted piece of data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct CipherText {
    /// Number only used once
    nonce: Vec<u8>,
    /// Data buffer
    data: Vec<u8>,
}
