use std::{
    io::{ErrorKind as IoErrorKind, Result as IoResult},
    path::{Path, PathBuf},
    pin::{pin, Pin},
};

use async_fs::{create_dir_all, File};
use bevy::{
    prelude::*,
    tasks::{
        futures_lite::{
            io::{BufReader, BufWriter}, AsyncRead,
            AsyncWrite,
        }, IoTaskPool,
        Task,
    },
    utils::{futures::check_ready, ConditionalSend, ConditionalSendFuture},
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
            create_dir_all(path.parent().unwrap()).await?;
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
        Self {
            settings_dir: dirs.preference_dir().into(),
            saves_dir: dirs.data_dir().join("saves"),
        }
    }
}

#[derive(Persist, Resource, Copy, Clone, Debug)]
#[persist(version = 0)]
pub struct InputKeyboardPref {
    /// Up-down-left-right, defaults to WSAD.
    pub movement: [KeyCode; 4],
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
    mut commands: Commands,
    storage: Res<LocalStorage>,
    mut channel: Local<(bool, Option<Task<IoResult<InputKeyboardPref>>>)>,
) {
    let (first, task) = &mut *channel;
    if !std::mem::replace(first, true) {
        *task = Some(IoTaskPool::get().spawn(storage.read_keyboard_pref()))
    }

    if let Some(task) = task &&
        task.is_finished()
    {
        let pref = check_ready(task)
            .expect("`is_finished()` implies Poll::Ready")
            .unwrap_or_else(|e| {
                if e.kind() != IoErrorKind::NotFound {
                    error!("Couldn't read keyboard input preference file: {e}")
                }

                default()
            });

        commands.insert_resource(pref);
        IoTaskPool::get().spawn(storage.write_keyboard_pref(pref)).detach()
    }
}
