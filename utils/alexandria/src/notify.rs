use crate::io::wire::{Decode, Encode};
use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer};
use std::task::Waker;
use std::{
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// Wake tasks on mutable accesses to the wrapped value
///
/// This can be used to transparently notify an asyncronous task that
/// it should, for example, check for more work in a queue or try
/// again to acquire a lock.
///
/// Most importantly, this type is serialisation transparent, meaning
/// it implements `Serialize`, `Deserialize` which is forwarded to the
/// implementations provided by `T`.
#[derive(Clone, Debug, Default)]
pub(crate) struct Notify<T> {
    inner: T,
    waker: Option<Waker>,
}

impl<T> Deref for Notify<T>
where
    T: DeserializeOwned + Serialize,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Notify<T>
where
    T: Encode<T> + Decode<T> + Serialize + DeserializeOwned,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.waker.as_ref().map(|w| w.wake_by_ref());
        &mut self.inner
    }
}

impl<T> Notify<T>
where
    T: Encode<T> + Decode<T> + Serialize + DeserializeOwned,
{
    /// Create an empty Notify handler
    pub(crate) fn new(inner: T) -> Self {
        Self { inner, waker: None }
    }

    /// Register a `Waker` if the wrapped value is updated
    ///
    /// This function will return the previous waker, if one existed.
    /// If `None` is returned, there was no previous waker, so be
    /// careful not to simply unwrap this value.  You may want to use
    /// `unwrap_none()`.
    #[allow(unused)]
    pub(crate) fn setup(ptr: &mut Notify<T>, waker: &Waker) {
        ptr.waker.replace(waker.clone());
    }

    /// Call wake on the waker, if there is a waker, yehaa!
    pub fn notify(ptr: &Notify<T>) {
        ptr.waker.as_ref().map(|w| w.wake_by_ref());
    }
}

impl<T> Serialize for Notify<T>
where
    T: Encode<T> + Decode<T> + Serialize + DeserializeOwned,
{
    fn serialize<S>(&self, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.inner.serialize(ser)
    }
}

impl<'de, T> Deserialize<'de> for Notify<T>
where
    T: Encode<T> + Decode<T> + Serialize + DeserializeOwned,
{
    fn deserialize<D>(de: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        T::deserialize(de).map(|inner| Self { inner, waker: None })
    }
}
