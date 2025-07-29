/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use anyhow::ensure;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(crate) struct SoundLibrary {
    path: PathBuf,
}

impl SoundLibrary {
    fn new(path: PathBuf) -> SoundLibrary {
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

pub(crate) struct AudioPlayer {
    sound_lib: SoundLibrary,
    _stream: OutputStream, // Hold reference to avoid sound playback from breaking!
    sink: Sink,
}

impl AudioPlayer {
    pub fn new(sounds_path: PathBuf) -> Result<AudioPlayer> {
        let sound_lib = SoundLibrary::new(sounds_path);

        let (_stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        Ok(AudioPlayer {
            sound_lib,
            _stream,
            sink,
        })
    }

    pub fn play(&self, name: &str) -> Result<()> {
        let filename = format!("{}.ogg", name);
        let source = self.sound_lib.load_sound(&filename)?;
        self.sink.append(source);
        self.sink.sleep_until_end();

        Ok(())
    }
}

fn load_source(path: &Path) -> Result<Decoder<BufReader<File>>> {
    let file = BufReader::new(File::open(path)?);
    Ok(Decoder::new(file)?)
}
