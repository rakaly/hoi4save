use std::fs::File;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

static DATA: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

/// See other impls for why this function exists
pub fn request_file<S: AsRef<str>>(input: S) -> File {
    let reffed = input.as_ref();
    let cache = Path::new("assets").join("saves").join(reffed);
    if cache.exists() {
        println!("cache hit: {}", reffed);
    } else {
        let guard = DATA.lock().unwrap();
        if cache.exists() {
            drop(guard);
            println!("cache hit: {}", reffed);
        } else {
            println!("cache miss: {}", reffed);
            let url = format!("https://cdn-dev.pdx.tools/hoi4-saves/{}", reffed);
            let mut resp = attohttpc::get(&url).send().unwrap();

            if !resp.is_success() {
                panic!("expected a 200 code from s3");
            } else {
                std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
                let mut f = std::fs::File::create(&cache).unwrap();
                std::io::copy(&mut resp, &mut f).unwrap();
            }
        }
    }

    std::fs::File::open(cache).unwrap()
}
