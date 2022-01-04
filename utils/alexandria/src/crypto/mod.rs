//! Provides more convenient crypto wrappers
#![allow(unused)]

pub(crate) mod aes;
pub(crate) mod bs;
pub(crate) mod pkcry;

mod engine;
pub(crate) use engine::{
    CryEngine, CryEngineHandle, CryReqPayload, CryRespPayload, ReqPayload, ResponsePayload,
};

mod hidden;
pub(crate) use hidden::Hid;

use crate::{
    error::{Error, Result},
    utils::Id,
};
use async_std::sync::Arc;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

/// A number that's only used once
pub(crate) type Nonce = [u8; 64];

/// An encrypted piece of data
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct CipherText {
    /// Number only used once
    pub(crate) nonce: Vec<u8>,
    /// Data buffer
    pub(crate) data: Vec<u8>,
}
