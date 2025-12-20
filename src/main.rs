mod error;
mod index;
mod tmsu;

use reqwest::StatusCode;
use serde::Deserialize;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::{Duration, Instant};
use walkdir::WalkDir;

use error::GeneralError;
use index::{load_index, save_index};
use tmsu::{tag_file, test_tmsu};

#[derive(Debug, Deserialize)]
struct ImagesEndpoint {
    pub image: Image,
}

#[derive(Debug, Deserialize)]
struct Image {
    pub duplicate_of: Option<u64>,
    pub tags: Option<Vec<String>>,
}

const INDEX_FNAME: &str = ".derpisync-index";
const API_LIMIT_PER_SEC: f64 = 0.9;

const AES_BOLD_GREEN: &str = "\x1b[1;32m";
const AES_BOLD_RED: &str = "\x1b[1;31m";
const AES_CLEAR: &str = "\x1b[0m";

fn query_image(id: u64, last_ts: &mut Instant) -> Result<ImagesEndpoint, GeneralError> {
    let api_delay = Duration::from_secs_f64(1.0 / API_LIMIT_PER_SEC);
    let remaining_delay = api_delay.saturating_sub(Instant::now() - *last_ts);
    sleep(remaining_delay);
    *last_ts = Instant::now();

    let query = format!("https://derpibooru.org/api/v1/json/images/{}", id);
    let mut answer = reqwest::blocking::get(&query)?;
    while !answer.status().is_success() {
        match answer.status() {
            StatusCode::NOT_IMPLEMENTED => {
                eprintln!("WARN: Got HTTP/501. Waiting for 6 secs to retry...");
                sleep(Duration::from_secs(6));
                answer = reqwest::blocking::get(&query)?;
            }
            status_code => {
                eprintln!(
                    "WARN: Got HTTP/{}. Waiting for 1 sec to retry...",
                    status_code
                );
                sleep(Duration::from_secs(1));
                answer = reqwest::blocking::get(&query)?;
            }
        }
    }
    let json = answer.text()?;
    Ok(serde_json::from_str(&json)?)
}

fn find_image_tags(
    id: u64,
    last_download_ts: &mut Instant,
) -> Result<Option<Vec<String>>, GeneralError> {
    let mut data = query_image(id, last_download_ts)?;
    while data.image.tags.is_none()
        && let Some(origin) = data.image.duplicate_of
    {
        data = query_image(origin, last_download_ts)?;
    }
    if let Some(tags) = data.image.tags {
        return Ok(Some(tags));
    } else {
        return Ok(None);
    }
}

fn id_from_filepath(filepath: &str, path_buf: &mut PathBuf) -> Option<u64> {
    path_buf.clear();
    path_buf.push(filepath);
    let file_name = path_buf.file_name()?.to_str()?;
    let id_str = if file_name.contains("__") {
        file_name.split("__").next().unwrap()
    } else {
        file_name.split(".").next().unwrap()
    };
    u64::from_str_radix(id_str, 10).ok()
}

fn should_omit(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with(".") || s.starts_with("db"))
        .unwrap_or(false)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let should_close = Arc::new(AtomicBool::new(false));
    let sc = should_close.clone();

    ctrlc::set_handler(move || {
        sc.store(true, Ordering::SeqCst);
    })?;

    if let Err(e) = test_tmsu() {
        eprintln!("{}", e);
        exit(1);
    }

    let mut btree = load_index(INDEX_FNAME)?;
    let mut path_buf = PathBuf::new();
    let mut last_download_ts = Instant::now();

    for entry in WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| f.file_type().is_file() && !should_omit(f))
    {
        if should_close.load(Ordering::SeqCst) {
            break;
        }

        let filepath_str = entry.path().to_str().unwrap();

        if btree.get(filepath_str).is_none() {
            let image_id = match id_from_filepath(filepath_str, &mut path_buf) {
                Some(id) => id,
                None => {
                    eprintln!(
                        "{}INFO{}: Doesn't look like derpibooru's downloaded image filename, skipping: {}",
                        AES_BOLD_GREEN, AES_CLEAR, filepath_str
                    );
                    continue;
                }
            };
            let tags = match find_image_tags(image_id, &mut last_download_ts) {
                Ok(Some(a)) => a,
                Ok(None) => {
                    eprintln!(
                        "{}INFO{}: Tags for {} are unavailable",
                        AES_BOLD_GREEN, AES_CLEAR, filepath_str
                    );
                    btree.insert(filepath_str.to_string());
                    continue;
                }
                Err(e) => {
                    eprintln!(
                        "{}ERROR{}: While getting tags for {}: {}",
                        AES_BOLD_RED, AES_CLEAR, filepath_str, e
                    );
                    continue;
                }
            };

            if let Err(e) = tag_file(filepath_str, tags) {
                eprintln!(
                    "{}ERROR{}: While tagging {}: {}",
                    AES_BOLD_RED, AES_CLEAR, filepath_str, e
                );
            } else {
                btree.insert(filepath_str.to_string());
                println!(
                    "{}INFO{}: {} is done",
                    AES_BOLD_GREEN, AES_CLEAR, filepath_str
                );
            }
        }
    }

    save_index(INDEX_FNAME, btree)?;
    Ok(())
}
