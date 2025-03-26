use std::{io::Result as IoResult, pin::Pin};

use bevy::{
    tasks::futures_lite::AsyncWrite,
    utils::{ConditionalSend, ConditionalSendFuture},
};

use crate::{persist::PersistWriter, w};

pub struct PersistSerializer<'a, W: AsyncWrite + ConditionalSend> {
    writer: Pin<&'a mut PersistWriter<W>>,
    buffer: Vec<u8>,
}

impl<'a, W: AsyncWrite + ConditionalSend> PersistSerializer<'a, W> {
    pub fn new(writer: Pin<&'a mut PersistWriter<W>>) -> Self {
        Self {
            writer,
            buffer: Vec::new(),
        }
    }
}

impl<'a, W: AsyncWrite + ConditionalSend> postcard::ser_flavors::Flavor for PersistSerializer<'a, W> {
    type Output = impl ConditionalSendFuture<Output = IoResult<()>>;

    fn try_extend(&mut self, data: &[u8]) -> postcard::Result<()> {
        self.buffer.extend_from_slice(data);
        Ok(())
    }

    fn try_push(&mut self, data: u8) -> postcard::Result<()> {
        self.buffer.push(data);
        Ok(())
    }

    fn finalize(self) -> postcard::Result<Self::Output> {
        let mut w = self.writer;
        let buf = self.buffer;

        Ok(async move {
            w!(w, usize: buf.len())?;
            w.write(&buf).await
        })
    }
}
