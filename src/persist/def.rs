use std::{
    borrow::Borrow,
    future::poll_fn,
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    pin::Pin,
    ptr::slice_from_raw_parts_mut,
};

use bevy::{
    tasks::futures_lite::{AsyncRead, AsyncWrite},
    utils::{ConditionalSend, ConditionalSendFuture},
};
use postcard::ser_flavors::Flavor;

use crate::persist::PersistSerializer;

#[macro_export]
macro_rules! r {
    ($reader:expr, $type:ty) => {
        <$type as $crate::persist::Persist>::read($reader.as_mut()).await
    };
}

#[macro_export]
macro_rules! w {
    ($writer:expr, $type:ty: $target:expr) => {
        $crate::persist::write::<_, $type>($writer.as_mut(), $target).await
    };
}

#[doc(hidden)]
#[inline(always)]
pub fn write<W: AsyncWrite + ConditionalSend, T: Persist>(
    writer: Pin<&mut PersistWriter<W>>,
    value: impl Borrow<T> + ConditionalSend,
) -> impl ConditionalSendFuture<Output = IoResult<()>> {
    async move {
        let value = value.borrow();
        T::write(value, writer).await
    }
}

pub struct PersistWriter<W: AsyncWrite + ConditionalSend>(W);
impl<W: AsyncWrite + ConditionalSend> PersistWriter<W> {
    pub fn new(writer: W) -> Self {
        Self(writer)
    }

    pub fn write(self: Pin<&mut Self>, mut bytes: &[u8]) -> impl ConditionalSendFuture<Output = IoResult<()>> {
        let mut writer = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        async move {
            while !bytes.is_empty() {
                let written = poll_fn(|ctx| writer.as_mut().poll_write(ctx, bytes)).await?;
                bytes = &bytes[written..];

                if written == 0 {
                    return Err(IoErrorKind::WriteZero.into())
                }
            }

            Ok(())
        }
    }

    pub fn ser(
        self: Pin<&mut Self>,
        acceptor: impl FnOnce(&mut postcard::Serializer<PersistSerializer<W>>) -> postcard::Result<()>,
    ) -> impl ConditionalSendFuture<Output = IoResult<()>> {
        let mut ser = postcard::Serializer {
            output: PersistSerializer::new(self),
        };

        let res = acceptor(&mut ser);
        async move {
            if let Err(e) = res {
                Err(IoError::new(IoErrorKind::InvalidData, e))
            } else {
                ser.output.finalize().unwrap().await?;
                Ok(())
            }
        }
    }
}

pub struct PersistReader<R: AsyncRead + ConditionalSend>(R);
impl<R: AsyncRead + ConditionalSend> PersistReader<R> {
    pub fn new(reader: R) -> Self {
        Self(reader)
    }

    pub fn read(self: Pin<&mut Self>, mut buffer: &mut [u8]) -> impl ConditionalSendFuture<Output = IoResult<()>> {
        let mut reader = unsafe { self.map_unchecked_mut(|s| &mut s.0) };
        async move {
            while !buffer.is_empty() {
                let read = poll_fn(|ctx| reader.as_mut().poll_read(ctx, &mut buffer)).await?;
                buffer = &mut buffer[read..];

                if read == 0 {
                    return Err(IoErrorKind::UnexpectedEof.into())
                }
            }

            Ok(())
        }
    }

    pub fn de<'de, T: 'de + ConditionalSend>(
        mut self: Pin<&mut Self>,
        buffer: &'de mut Vec<u8>,
        acceptor: impl FnOnce(&mut postcard::Deserializer<'de, postcard::de_flavors::Slice<'de>>) -> postcard::Result<T>
        + ConditionalSend,
    ) -> impl ConditionalSendFuture<Output = IoResult<T>> {
        async move {
            let len = r!(self, usize)?;
            buffer.reserve_exact(len);

            let off = buffer.len();
            let slice = unsafe {
                let ptr = buffer.as_mut_ptr().add(off);
                ptr.write_bytes(0, len);

                buffer.set_len(off + len);
                &mut *slice_from_raw_parts_mut(ptr, len)
            };

            self.read(slice).await?;

            let mut de = postcard::Deserializer::from_bytes(slice);
            acceptor(&mut de).map_err(|e| IoError::new(IoErrorKind::InvalidData, e))
        }
    }
}

pub trait Persist: ConditionalSend + Sync + Clone {
    fn read<R: AsyncRead + ConditionalSend>(
        r: Pin<&mut PersistReader<R>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<Self>>;

    fn write<W: AsyncWrite + ConditionalSend>(
        &self,
        w: Pin<&mut PersistWriter<W>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<()>>;
}

macro_rules! impl_persist_integer {
    ($($name:ty)*) => {
        $(
            impl Persist for $name {
                async fn read<R: AsyncRead + ConditionalSend>(
                    r: Pin<&mut PersistReader<R>>,
                ) -> IoResult<Self> {
                    let mut bytes = [0; size_of::<Self>()];
                    r.read(&mut bytes).await?;

                    Ok(Self::from_le_bytes(bytes))
                }

                async fn write<W: AsyncWrite + ConditionalSend>(&self, w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
                    w.write(&self.to_le_bytes()).await
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
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let num = r!(r, u32)?;
        usize::try_from(num).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                format!("Index exceeded `usize::MAX`: {num} > {}", usize::MAX),
            )
        })
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        match u32::try_from(*self) {
            Ok(num) => w!(w, u32: num),
            Err(..) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("Index exceeded `u32::MAX`: {self} > {}", u32::MAX),
            )),
        }
    }
}

// Use `i32` for `isize` to ensure consistent save files across machines of different architectures.
impl Persist for isize {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let num = r!(r, i32)?;
        isize::try_from(num).map_err(|_| {
            IoError::new(
                IoErrorKind::InvalidData,
                format!("Index exceeded `isize::MAX`: {num} > {}", isize::MAX),
            )
        })
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        match i32::try_from(*self) {
            Ok(num) => w!(w, i32: num),
            Err(..) => Err(IoError::new(
                IoErrorKind::InvalidInput,
                format!("Index exceeded `i32::MAX`: {self} > {}", i32::MAX),
            )),
        }
    }
}

impl Persist for String {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let len = r!(r, usize)?;
        let mut this = vec![0; len];

        r.read(&mut this).await?;
        String::from_utf8(this).map_err(|e| IoError::new(IoErrorKind::InvalidData, e))
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w!(w, usize: self.len())?;
        w.write(self.as_bytes()).await
    }
}

impl<T: Persist> Persist for Vec<T> {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        let len = r!(r, usize)?;

        let mut this = Vec::with_capacity(len);
        for _ in 0..len {
            this.push(r!(r, T)?)
        }

        Ok(this)
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w!(w, usize: self.len())?;
        for item in &self[..] {
            w!(w, T: item)?
        }

        Ok(())
    }
}

pub trait PersistVersion<const VERSION: u16>: Persist {
    fn read_versioned<R: AsyncRead + ConditionalSend>(
        r: Pin<&mut PersistReader<R>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<Self>>;

    fn write_versioned<W: AsyncWrite + ConditionalSend>(
        &self,
        w: Pin<&mut PersistWriter<W>>,
    ) -> impl ConditionalSendFuture<Output = IoResult<()>>;
}
