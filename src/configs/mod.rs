pub mod launcher;

use std::{path::PathBuf, fs::{OpenOptions, File}, io::Write};

use serde::Serialize;

struct ConfigFile(bool, PathBuf);

impl ConfigFile {
  pub fn new(path: PathBuf) -> Self {
    Self(path.exists(), path)
  }
}

pub trait Config {
  fn write(&self, path: PathBuf) -> Result<(), std::io::Error>
  where Self: Serialize
  {
    let conf: ConfigFile = ConfigFile::new(path);
    let mut file: File = std::fs::File::create(conf.1.clone()).unwrap();
    
    let _ = serde_json::to_writer_pretty(&mut file, &self);

    println!("created config");
    Ok(())
  }

  fn overwrite(&self, path: PathBuf)
  where Self: Serialize
  {
    let conf: ConfigFile = ConfigFile::new(path);
    match conf.0 {
      true => {
        let mut file = OpenOptions::new()
          .write(true)
          .truncate(true)
          .open(conf.1)
          .unwrap();
    
        let _ = serde_json::to_writer_pretty(&mut file, &self);
      },
      false => {
        self.write(conf.1).unwrap()
      }
    }
  }

  fn read_config(&self, path: PathBuf) -> Result<Self, ()>
  where Self: Sized + for<'de> serde::Deserialize<'de> + Serialize
  {
    let conf: ConfigFile = ConfigFile::new(path);
    if conf.0 {
      let f = std::fs::File::open(conf.1).expect("Could not open file");
      let read: Self = serde_json::from_reader(f).expect("Could not read values");
      return Ok(read);
    } else {
      let _ = self.overwrite(conf.1);
      return Err(());
    }
  }
}

