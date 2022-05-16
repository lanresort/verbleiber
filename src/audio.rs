/*
 * Copyright 2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use rodio::{Decoder, Sink};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub(crate) struct Player<'a> {
    dir: PathBuf,
    sink: &'a Sink,
}

impl<'a> Player<'a> {
    pub fn new(dir: PathBuf, sink: &Sink) -> Player {
        Player {
            dir: dir,
            sink: sink,
        }
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
