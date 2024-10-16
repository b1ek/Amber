use include_dir::{include_dir, Dir};
pub const STDLIB: Dir = include_dir!("src/std");

pub fn resolve<T: Into<String>>(path: T) -> Option<String> {
    let path = path.into();

    if let Some(module) = STDLIB.get_entry(path + ".ab") {
        let module = module.as_file()?.contents_utf8()?.to_string();
        Some(module)
    } else {
        None
    }
}
