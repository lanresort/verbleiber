/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use anyhow::ensure;
use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink};
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
    _output_stream: OutputStream, // Hold reference to keep sound playback working!
    sink: Sink,
}

impl AudioPlayer {
    pub fn new(sounds_path: PathBuf) -> Result<AudioPlayer> {
        let sound_lib = SoundLibrary::new(sounds_path);

        let output_stream = OutputStreamBuilder::open_default_stream()?;
        let sink = Sink::connect_new(output_stream.mixer());

        Ok(AudioPlayer {
            sound_lib,
            _output_stream: output_stream,
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
