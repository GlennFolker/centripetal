use std::{io::Result as IoResult, pin::Pin};

use bevy::{
    prelude::*,
    tasks::futures_lite::{AsyncRead, AsyncWrite},
    utils::ConditionalSend,
};
use serde::{de::Deserialize, ser::Serialize};

use crate::persist::{Persist, PersistReader, PersistWriter};

impl Persist for KeyCode {
    async fn read<R: AsyncRead + ConditionalSend>(r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        r.de(&mut Vec::new(), |de| KeyCode::deserialize(de)).await
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w.ser(|ser| self.serialize(ser)).await
    }
}
