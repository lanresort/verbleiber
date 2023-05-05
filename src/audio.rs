/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::ensure;
use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(crate) struct SoundLibrary {
    path: PathBuf,
}

impl SoundLibrary {
    pub fn new(path: PathBuf) -> SoundLibrary {
        SoundLibrary { path }
    }

    fn load_sound(&self, filename: &str) -> Result<Decoder<BufReader<File>>> {
        let path = self.path.join(filename);
        ensure!(
            &path.exists(),
            "Sound file {} does not exist.",
            path.display()
        );

        let source = load_source(&path)?;

        Ok(source)
    }
}

pub(crate) struct Player {
    sound_lib: SoundLibrary,
    sink: Sink,
}

impl Player {
    pub fn new(sounds_path: PathBuf) -> Player {
        let sound_lib = SoundLibrary::new(sounds_path);

        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.sleep_until_end();

        Player { sound_lib, sink }
    }

    pub fn play(&self, filename: &str) -> Result<()> {
        let source = self.sound_lib.load_sound(filename)?;
        self.sink.append(source);

        Ok(())
    }
}

fn load_source(path: &Path) -> Result<Decoder<BufReader<File>>> {
    let file = BufReader::new(File::open(path)?);
    Ok(Decoder::new(file)?)
}
