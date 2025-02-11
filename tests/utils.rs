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
            let url = format!(
                "https://hoi4saves-test-cases.s3.us-west-002.backblazeb2.com/{}",
                reffed
            );
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

/// See other impls for the reason for this function
pub fn inflate(file: File) -> Vec<u8> {
    let mut buf = vec![0u8; rawzip::RECOMMENDED_BUFFER_SIZE];
    let archive = rawzip::ZipArchive::from_file(file, &mut buf).unwrap();
    let mut entries = archive.entries(&mut buf);
    let entry = entries.next_entry().unwrap().unwrap();
    let wayfinder = entry.wayfinder();
    let mut output = Vec::with_capacity(wayfinder.uncompressed_size_hint() as usize);
    let zip_entry = archive.get_entry(wayfinder).unwrap();
    let inflater = flate2::read::DeflateDecoder::new(zip_entry.reader());
    let mut verifier = zip_entry.verifying_reader(inflater);
    std::io::copy(&mut verifier, &mut output).unwrap();
    output
}
