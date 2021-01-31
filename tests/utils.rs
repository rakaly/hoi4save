use std::{fs, io::Cursor};
use std::path::Path;
use std::io::Read;

/// See other impls for the reason for this function
pub fn retrieve<S: AsRef<str>>(input: S) -> Vec<u8> {
    let reffed = input.as_ref();
    let cache = Path::new("assets").join("saves").join(reffed);
    if cache.exists() {
        println!("cache hit: {}", reffed);
        fs::read(cache).unwrap()
    } else {
        println!("cache miss: {}", reffed);
        let url = format!(
            "https://hoi4saves-test-cases.s3.us-west-002.backblazeb2.com/{}",
            reffed
        );
        let resp = attohttpc::get(&url).send().unwrap();

        if !resp.is_success() {
            panic!("expected a 200 code from s3");
        } else {
            let data = resp.bytes().unwrap();
            std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
            std::fs::write(&cache, &data).unwrap();
            data
        }
    }
}

/// See other impls for the reason for this function
pub fn request<S: AsRef<str>>(input: S) -> Vec<u8> {
    let data = retrieve(input);
    let reader = Cursor::new(&data[..]);
    let mut zip = zip::ZipArchive::new(reader).unwrap();
    let mut zip_file = zip.by_index(0).unwrap();
    let mut buffer = Vec::with_capacity(0);
    zip_file.read_to_end(&mut buffer).unwrap();
    buffer
}
