use std::{io::Result as IoResult, pin::Pin};

use bevy::{
    prelude::*,
    tasks::futures_lite::{AsyncRead, AsyncWrite},
    utils::ConditionalSend,
};

use crate::{
    de,
    persist::{Persist, PersistReader, PersistWriter},
    ser,
};

impl Persist for KeyCode {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        de!(r, KeyCode)
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        ser!(w, KeyCode: self)
    }
}
