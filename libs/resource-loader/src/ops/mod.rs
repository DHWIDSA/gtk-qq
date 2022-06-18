pub mod avatar;
pub mod database;
pub mod template;
use std::{fs::create_dir_all, io, path::Path};

pub trait GetPath {
    fn get_path() -> &'static Path;

    fn get_and_create_path() -> io::Result<&'static Path> {
        let path = <Self as GetPath>::get_path();

        #[cfg(feature = "logger")]
        log::debug!("get and create path {:?}", path);

        create_dir_all(path)?;
        Ok(path)
    }

    fn get_and_do_action(action: DirAction) -> io::Result<&'static Path> {
        match action {
            DirAction::CreateAll => <Self as GetPath>::get_and_create_path(),
            DirAction::None => Ok(<Self as GetPath>::get_path()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DirAction {
    CreateAll,
    None,
}
