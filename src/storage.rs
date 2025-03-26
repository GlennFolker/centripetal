use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind, ErrorKind, Result as IoResult},
    path::{Path, PathBuf},
    pin::{pin, Pin},
};

use async_channel::Receiver;
use async_fs::File;
use bevy::{
    prelude::*,
    tasks::{
        futures_lite::{
            io::{BufReader, BufWriter}, AsyncRead,
            AsyncWrite,
        },
        IoTaskPool,
    },
    utils::{ConditionalSend, ConditionalSendFuture},
};
use directories::ProjectDirs;

use crate::{
    persist::{Persist, PersistReader, PersistVersion, PersistWriter},
    r, w,
};

pub enum Storage {
    Settings,
    Saves,
}

#[derive(Resource, Debug)]
pub struct LocalStorage {
    settings_dir: PathBuf,
    saves_dir: PathBuf,
}

impl LocalStorage {
    pub fn reader<P: AsRef<Path>>(
        &self,
        storage: Storage,
        file: P,
    ) -> impl ConditionalSendFuture<Output = IoResult<PersistReader<impl AsyncRead + ConditionalSend + use<P>>>> + use<P>
    {
        let path = match storage {
            Storage::Settings => &self.settings_dir,
            Storage::Saves => &self.saves_dir,
        }
        .join(file);

        async move {
            let file = File::open(path).await?;
            Ok(PersistReader::new(BufReader::new(file)))
        }
    }

    pub fn writer<P: AsRef<Path>>(
        &self,
        storage: Storage,
        file: P,
    ) -> impl ConditionalSendFuture<Output = IoResult<PersistWriter<impl AsyncWrite + ConditionalSend + use<P>>>> + use<P>
    {
        let path = match storage {
            Storage::Settings => &self.settings_dir,
            Storage::Saves => &self.saves_dir,
        }
        .join(file);

        async move {
            let file = File::create(path).await?;
            Ok(PersistWriter::new(BufWriter::new(file)))
        }
    }

    pub fn read_keyboard_pref(&self) -> impl ConditionalSendFuture<Output = IoResult<InputKeyboardPref>> + use<> {
        let reader = self.reader(Storage::Settings, "keyboard.pref");
        async move {
            let mut r = pin!(reader.await?);
            r!(r, InputKeyboardPref)
        }
    }

    pub fn write_keyboard_pref(&self, pref: InputKeyboardPref) -> impl ConditionalSendFuture<Output = IoResult<()>> + use<> {
        let writer = self.writer(Storage::Settings, "keyboard.pref");
        async move {
            let mut w = pin!(writer.await?);
            w!(w, InputKeyboardPref: pref)?;
            w.close().await
        }
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        let dirs = ProjectDirs::from("com.github", "gygl", "Centripetal").expect("couldn't get project data directories");
        let storage = Self {
            settings_dir: dirs.preference_dir().into(),
            saves_dir: dirs.data_dir().join("saves"),
        };

        std::fs::create_dir_all(&storage.settings_dir).unwrap_or_else(|e| panic!("Couldn't create settings directory: {e}"));
        std::fs::create_dir_all(&storage.saves_dir).unwrap_or_else(|e| panic!("Couldn't create saves directory: {e}"));
        storage
    }
}

#[derive(Resource, Copy, Clone, Debug)]
pub struct InputKeyboardPref {
    /// Up-down-left-right, defaults to WSAD.
    pub movement: [KeyCode; 4],
}

impl Persist for InputKeyboardPref {
    async fn read<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        match r!(r, u16)? {
            0 => <Self as PersistVersion<0>>::read_versioned(r).await,
            v => Err(IoError::new(IoErrorKind::InvalidData, format!("Invalid version: {v}."))),
        }
    }

    async fn write<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w!(w, u16: 0)?;
        <Self as PersistVersion<0>>::write_versioned(self, w).await
    }
}

impl PersistVersion<0> for InputKeyboardPref {
    async fn read_versioned<R: AsyncRead + ConditionalSend>(mut r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        Ok(Self {
            movement: r!(r, [KeyCode; 4])?,
        })
    }

    async fn write_versioned<W: AsyncWrite + ConditionalSend>(&self, mut w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w!(w, [KeyCode; 4]: self.movement)
    }
}

impl Default for InputKeyboardPref {
    fn default() -> Self {
        Self {
            movement: [KeyCode::KeyW, KeyCode::KeyS, KeyCode::KeyA, KeyCode::KeyD],
        }
    }
}

pub struct StoragePlugin;
impl Plugin for StoragePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LocalStorage>()
            .init_resource::<InputKeyboardPref>()
            .add_systems(Update, load_input_pref);
    }
}

fn load_input_pref(
    storage: Res<LocalStorage>,
    mut pref: ResMut<InputKeyboardPref>,
    mut first: Local<bool>,
    mut channel: Local<Option<Receiver<IoResult<(InputKeyboardPref, bool)>>>>,
) {
    if !std::mem::replace(&mut first, true) {
        let (sender, receiver) = async_channel::bounded(1);
        *channel = Some(receiver);

        let sys = storage.read_keyboard_pref();
        IoTaskPool::get()
            .spawn(async move {
                let result = match sys.await {
                    Ok(pref) => Ok((pref, false)),
                    Err(e) if e.kind() == ErrorKind::NotFound => {
                        info!("Keyboard input preference file not found; creating a new one.");
                        Ok((default(), true))
                    }
                    Err(e) => Err(e),
                };

                _ = sender.send(result).await;
            })
            .detach()
    }

    if let Some(recv) = channel.as_ref() &&
        let Ok(new_pref) = recv.try_recv()
    {
        *channel = None;
        match new_pref {
            Ok((new_pref, create_new)) => {
                *pref = new_pref;
                if create_new {
                    let sys = storage.write_keyboard_pref(new_pref);
                    IoTaskPool::get()
                        .spawn(async move {
                            if let Err(e) = sys.await {
                                error!("Couldn't write keyboard input preference file: {e}")
                            } else {
                                info!("Successfully written keyboard input preference file!")
                            }
                        })
                        .detach()
                }
            }
            Err(e) => error!("Couldn't load keyboard input preference: {e}"),
        }
    }
}
