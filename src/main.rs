mod error;
mod index;
mod tmsu;

use reqwest::StatusCode;
use serde::Deserialize;
use std::io::stdin;
use std::path::PathBuf;
use std::process::exit;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::sleep;
use std::time::Duration;

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

fn query_image(id: u64) -> Result<ImagesEndpoint, GeneralError> {
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

fn find_image_tags(id: u64) -> Result<Option<Vec<String>>, GeneralError> {
    let mut data = query_image(id)?;
    while data.image.tags.is_none()
        && let Some(origin) = data.image.duplicate_of
    {
        data = query_image(origin)?;
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

    let stdin = stdin();
    let mut line = String::new();
    let mut path_buf = PathBuf::new();

    while !should_close.load(Ordering::SeqCst) && stdin.read_line(&mut line)? != 0 {
        let filepath = line.trim_end();

        if btree.get(filepath).is_none() {
            let image_id = match id_from_filepath(filepath, &mut path_buf) {
                Some(id) => id,
                None => {
                    println!(
                        "INFO: Doesn't look like derpibooru's downloaded image filename, skipping: {}",
                        filepath
                    );
                    line.clear();
                    continue;
                }
            };
            let tags = match find_image_tags(image_id) {
                Ok(Some(a)) => a,
                Ok(None) => {
                    eprintln!("INFO: Tags for {} are unavailable", filepath);
                    btree.insert(filepath.to_string());
                    line.clear();
                    continue;
                }
                Err(e) => {
                    eprintln!("ERROR: While getting tags for {}: {}", filepath, e);
                    line.clear();
                    continue;
                }
            };

            if let Err(e) = tag_file(filepath, tags) {
                eprintln!("While tagging {}: {}", filepath, e);
            } else {
                btree.insert(filepath.to_string());
                println!("INFO: {} is done", filepath);
            }
        }

        line.clear();
    }

    save_index(INDEX_FNAME, btree)?;
    Ok(())
}
