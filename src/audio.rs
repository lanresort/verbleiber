/*
 * Copyright 2022-2023 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use rodio::{Decoder, OutputStream, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(crate) struct Player {
    dir: PathBuf,
    sink: Sink,
}

impl Player {
    pub fn new(dir: PathBuf) -> Player {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        sink.sleep_until_end();

        Player { dir, sink }
    }

    pub fn play(&self, filename: &str) -> Result<()> {
        let path = self.dir.join(filename);
        if !&path.exists() {
            eprintln!("Sound file {} does not exist.", path.display());
            return Ok(());
        }

        let source = load_source(&path)?;
        self.sink.append(source);

        Ok(())
    }
}

fn load_source(path: &Path) -> Result<Decoder<BufReader<File>>> {
    let file = BufReader::new(File::open(path)?);
    Ok(Decoder::new(file)?)
}
