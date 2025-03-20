use std::{
    future::poll_fn,
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    pin::Pin,
    task::{ready, Poll},
};

use bevy::{
    tasks::futures_lite::{AsyncRead, AsyncWrite},
    utils::{ConditionalSend, ConditionalSendFuture},
};

pub struct PersistWriter<W: AsyncWrite + ConditionalSend>(W);
impl<W: AsyncWrite + ConditionalSend> PersistWriter<W> {
    pub fn write(self: Pin<&mut Self>, mut bytes: &[u8]) -> impl ConditionalSendFuture<Output = IoResult<()>> + Sized {
        let mut writer = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        poll_fn(move |ctx| {
            while !bytes.is_empty() {
                let written = ready!(writer.as_mut().poll_write(ctx, bytes))?;
                bytes = &bytes[written..];

                if written == 0 {
                    return Poll::Ready(Err(IoErrorKind::WriteZero.into()))
                }
            }

            Poll::Ready(Ok(()))
        })
    }
}

pub struct PersistReader<R: AsyncRead + ConditionalSend>(R);
impl<R: AsyncRead + ConditionalSend> PersistReader<R> {
    pub fn read(self: Pin<&mut Self>, buffer: &mut [u8]) -> impl ConditionalSendFuture<Output = IoResult<()>> + Sized {
        let mut reader = unsafe { self.map_unchecked_mut(|s| &mut s.0) };

        let mut offset = 0;
        poll_fn(move |ctx| {
            while !buffer.is_empty() {
                let read = ready!(reader.as_mut().poll_read(ctx, &mut buffer[offset..]))?;
                offset += read;

                if read == 0 {
                    return Poll::Ready(Err(IoErrorKind::UnexpectedEof.into()))
                }
            }

            Poll::Ready(Ok(()))
        })
    }
}

pub trait Persist: ConditionalSend + Sync + Sized {
    fn read<R: AsyncRead + ConditionalSend>(
        reader: Pin<&mut PersistReader<R>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<Self>>;

    fn write<W: AsyncWrite + ConditionalSend>(
        &self,
        writer: Pin<&mut PersistWriter<W>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<()>>;
}

macro_rules! impl_persist_integer {
    ($($name:ty)*) => {
        $(
            impl Persist for $name {
                async fn read<R: AsyncRead + ConditionalSend>(
                    reader: Pin<&mut PersistReader<R>>,
                ) -> IoResult<Self> {
                    let mut bytes = [0; size_of::<Self>()];
                    reader.read(&mut bytes).await?;

                    Ok(Self::from_le_bytes(bytes))
                }

                async fn write<W: AsyncWrite + ConditionalSend>(&self, writer: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
                    writer.write(&self.to_le_bytes()).await
                }
            }
        )*
    };
}

impl_persist_integer!(
    u8 u16 u32 u64 u128
    i8 i16 i32 i64 i128
);

// Use `u32` for `usize` to ensure consistent save files across machines of different architectures.
impl Persist for usize {
    async fn read<R: AsyncRead + ConditionalSend>(reader: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let num = u32::read(reader).await?;
        usize::try_from(num).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                format!("Index exceeded `usize::MAX`: {num} > {}", usize::MAX),
            )
        })
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, writer: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        match u32::try_from(*self) {
            Ok(num) => num.write(writer).await,
            Err(..) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("Index exceeded `u32::MAX`: {self} > {}", u32::MAX),
            )),
        }
    }
}

// Use `i32` for `isize` to ensure consistent save files across machines of different architectures.
impl Persist for isize {
    async fn read<R: AsyncRead + ConditionalSend>(reader: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let num = i32::read(reader).await?;
        isize::try_from(num).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                format!("Index exceeded `isize::MAX`: {num} > {}", isize::MAX),
            )
        })
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, writer: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        match i32::try_from(*self) {
            Ok(num) => num.write(writer).await,
            Err(..) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("Index exceeded `i32::MAX`: {self} > {}", i32::MAX),
            )),
        }
    }
}

impl Persist for String {
    async fn read<R: AsyncRead + ConditionalSend>(mut reader: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let len = usize::read(reader.as_mut()).await?;
        let mut this = vec![0; len];

        reader.read(&mut this).await?;
        String::from_utf8(this).map_err(|e| IoError::new(IoErrorKind::InvalidData, e))
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut writer: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        self.len().write(writer.as_mut()).await?;
        writer.write(self.as_bytes()).await
    }
}

impl<T: Persist> Persist for Vec<T> {
    async fn read<R: AsyncRead + ConditionalSend>(mut reader: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let len = usize::read(reader.as_mut()).await?;

        let mut this = Vec::with_capacity(len);
        for _ in 0..len {
            this.push(T::read(reader.as_mut()).await?)
        }

        Ok(this)
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut writer: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        self.len().write(writer.as_mut()).await?;
        for item in &self[..] {
            item.write(writer.as_mut()).await?
        }

        Ok(())
    }
}

pub trait PersistVersion<const VERSION: u16>: Persist {
    fn read_versioned<R: AsyncRead + ConditionalSend>(
        reader: Pin<&mut PersistReader<R>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<Self>>;

    fn write_versioned<W: AsyncWrite + ConditionalSend>(
        &self,
        writer: Pin<&mut PersistWriter<W>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<()>>;
}
