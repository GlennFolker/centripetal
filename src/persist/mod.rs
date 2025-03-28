use std::{io::Result as IoResult, pin::Pin};

use bevy::{
    prelude::*,
    tasks::futures_lite::{AsyncRead, AsyncWrite},
    utils::ConditionalSend,
};

mod def;
mod serde;
pub use def::*;
pub use serde::*;

use crate::{de, ser};

impl Persist for KeyCode {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        de!(r, KeyCode)
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        ser!(w, KeyCode: self)
    }
}
