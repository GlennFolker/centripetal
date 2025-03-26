use std::{
    io::{Error as IoError, ErrorKind as IoErrorKind, Result as IoResult},
    path::{Path, PathBuf},
    pin::{pin, Pin},
};

use async_fs::File;
use bevy::{
    prelude::*,
    tasks::{
        futures_lite::{
            io::{BufReader, BufWriter}, AsyncRead,
            AsyncWrite,
        }, IoTaskPool,
        Task,
    },
    utils::{ConditionalSend, ConditionalSendFuture},
};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{
    persist::{Persist, PersistReader, PersistVersion, PersistWriter},
    r, w,
};

pub enum Storage {
    Settings,
    Saves,
}

#[derive(Resource)]
pub struct LocalStorage {
    settings_dir: PathBuf,
    saves_dir: PathBuf,
}

impl LocalStorage {
    pub fn reader(
        &self,
        storage: Storage,
        file: impl AsRef<Path>,
    ) -> impl ConditionalSendFuture<Output = IoResult<PersistReader<impl AsyncRead + ConditionalSend>>> + use<> {
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

    pub fn writer(
        &self,
        storage: Storage,
        file: impl AsRef<Path>,
    ) -> impl ConditionalSendFuture<Output = IoResult<PersistWriter<impl AsyncWrite + ConditionalSend>>> {
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

    pub fn write_keyboard_pref(&self, pref: &InputKeyboardPref) -> impl ConditionalSendFuture<Output = IoResult<()>> {
        async move {
            let mut w = pin!(self.writer(Storage::Settings, "keyboard.pref").await?);
            w!(w, InputKeyboardPref: pref)
        }
    }
}

impl Default for LocalStorage {
    fn default() -> Self {
        let dirs = ProjectDirs::from("com.github", "gygl", "Centripetal").expect("couldn't get project data directories");
        Self {
            settings_dir: dirs.preference_dir().into(),
            saves_dir: dirs.data_dir().join("saves"),
        }
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
    async fn read_versioned<R: AsyncRead + ConditionalSend>(r: Pin<&mut PersistReader<R>>) -> IoResult<Self> {
        Ok(Self {
            movement: r.de(&mut Vec::new(), |de| <[KeyCode; 4]>::deserialize(de)).await?,
        })
    }

    async fn write_versioned<W: AsyncWrite + ConditionalSend>(&self, w: Pin<&mut PersistWriter<W>>) -> IoResult<()> {
        w.ser(|se| self.movement.serialize(se)).await
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
    mut first: Local<bool>,
    mut task: Local<Option<Task<IoResult<InputKeyboardPref>>>>,
) {
    if !std::mem::replace(&mut first, true) {
        let sys = storage.read_keyboard_pref();
        *task = Some(IoTaskPool::get().spawn(async move { sys.await }));
    }
}
