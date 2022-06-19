mod ops;
#[macro_use]
mod utils;
mod configs;
mod static_data;

pub use configs::Config;

pub use static_data::{load_from_file, save_config, set_config};

pub use ops::{
    avatar::{Group as AvatarGroup, User as AvatarUser},
    database::SqlDataBase,
    temporary::{CaptchaQrCode, TempDir},
    AsyncCreatePath, AsyncLoadResource, DirAction, GetPath, SyncCreatePath, SyncLoadResource,
};
