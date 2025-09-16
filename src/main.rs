use serde::Deserialize;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};

#[derive(Debug, Deserialize)]
struct ImagesEndpoint {
    pub image: Image,
}

#[derive(Debug, Deserialize)]
struct Image {
    pub duplicate_of: Option<u64>,
    pub tags: Option<Vec<String>>,
}

const ERR_CHILD_EXEC: &str = "ERROR: Unknown error during child execution";
const ERR_INV_FNAME: &str = "ERROR: Invalid filename";
const ERR_INV_JSON: &str = "ERROR: Invalid json";

fn query_image(id: u64) -> ImagesEndpoint {
    let query = format!("https://derpibooru.org/api/v1/json/images/{}", id);
    let json = reqwest::blocking::get(query)
        .expect("ERROR: Response receival error")
        .text()
        .expect("ERROR: Getting responce text error");
    serde_json::from_str(&json).expect(ERR_INV_JSON)
}

fn test_tmsu_availability() {
    let tmsu_ver_status = Command::new("tmsu")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .expect(ERR_CHILD_EXEC);
    if !tmsu_ver_status.success() {
        eprintln!("ERROR: You are likely doesn't have tmsu installed");
        exit(1);
    }
}

fn test_tmsu_db_availability() {
    let tmsu_info_status = Command::new("tmsu")
        .arg("info")
        .stdout(Stdio::null())
        .status()
        .expect(ERR_CHILD_EXEC);
    if !tmsu_info_status.success() {
        eprintln!("ERROR: There is no tmsu database");
        exit(1);
    }
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        eprintln!("USAGE: derpisync FILENAME_OUT_OF_ID_AND_EXTENSION");
        exit(1);
    }
    let _ = args.next(); // Program name
    let file_str = args.next().unwrap();
    let file_path = PathBuf::from(&file_str);
    let file_name = file_path
        .file_name()
        .expect(ERR_INV_FNAME)
        .to_str()
        .expect(ERR_INV_FNAME);
    let image_id = file_name.split(".").next().expect(ERR_INV_FNAME);
    let image_id = u64::from_str_radix(&image_id, 10).expect("ERROR: Invalid image id");

    test_tmsu_availability();
    test_tmsu_db_availability();

    let mut img = query_image(image_id);
    if img.image.tags.is_none() {
        if let Some(original) = img.image.duplicate_of {
            img = query_image(original);
        } else {
            println!("INFO: Tags are unavaliable for this image");
            exit(0);
        }
    }

    let mut tmsu_tag = Command::new("tmsu");
    tmsu_tag.arg("tag");
    tmsu_tag.arg(file_str);
    tmsu_tag.args(img.image.tags.unwrap());
    tmsu_tag.stdout(Stdio::null());

    let tmsu_tag_status = tmsu_tag.status().expect(ERR_CHILD_EXEC);
    if !tmsu_tag_status.success() {
        eprintln!("ERROR: Tagging process was unsuccessful");
        exit(1);
    }
}
