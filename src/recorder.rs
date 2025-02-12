use std::{
    fs::{OpenOptions, create_dir_all},
    io,
    path::PathBuf,
};

use log::error;
use serde::Serialize;

use crate::Args;

#[derive(Clone)]
pub struct Recorder {
    base_path: PathBuf,
}

impl Recorder {
    pub fn new(args: &Args) -> Recorder {
        Recorder {
            base_path: args.record_path.clone(),
        }
    }

    pub fn record(&self, game_name: &str, configuration: &str, record: impl Serialize) {
        let _ = self
            .record_inner(game_name, configuration, record)
            .map_err(|e| error!("{e}"));
    }

    pub fn record_inner(
        &self,
        game_name: &str,
        configuration: &str,
        record: impl Serialize,
    ) -> Result<(), io::Error> {
        let mut path = self.base_path.clone();
        path.push(game_name);
        let _ = create_dir_all(&path);
        path.push(format!("{configuration}.csv"));
        let file = OpenOptions::new().append(true).create(true).open(path)?;
        let mut writer = csv::WriterBuilder::new()
            .has_headers(false)
            .from_writer(file);
        writer.serialize(record)?;
        writer.flush()?;
        Ok(())
    }
}
