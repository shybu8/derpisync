mod tmsu;

use serde::Deserialize;
use std::env;
use std::io::stdin;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut args = env::args();
    // if args.len() != 2 {
    //     eprintln!("USAGE: derpisync FILENAME_OUT_OF_ID_AND_EXTENSION");
    //     exit(1);
    // }

    // let _ = args.next(); // Program name
    // let file_str = args.next().unwrap();
    // let file_path = PathBuf::from(&file_str);
    // let file_name = file_path
    //     .file_name()
    //     .expect(ERR_INV_FNAME)
    //     .to_str()
    //     .expect(ERR_INV_FNAME);
    // let image_id = file_name.split(".").next().expect(ERR_INV_FNAME);
    // let image_id = u64::from_str_radix(&image_id, 10).expect("ERROR: Invalid image id");

    if let Err(e) = test_tmsu() {
        eprintln!("{}", e);
        exit(1);
    }

    let stdin = stdin();
    let mut line = String::new();
    let mut path_buf = PathBuf::new();
    let mut succ_flag: bool;

    while stdin.read_line(&mut line)? != 0 {
        succ_flag = true;
        let filepath = line.trim_end();
        path_buf.push(filepath);
        let file_name = path_buf
            .file_name()
            .expect(ERR_INV_FNAME)
            .to_str()
            .expect(ERR_INV_FNAME);
        let image_id = file_name.split(".").next().expect(ERR_INV_FNAME);
        let image_id = u64::from_str_radix(&image_id, 10).expect("ERROR: Invalid image id");
        let mut img = query_image(image_id);
        if img.image.tags.is_none() {
            if let Some(original) = img.image.duplicate_of {
                img = query_image(original);
            } else {
                println!("INFO: Tags are unavaliable for {}", line);
                succ_flag = false;
            }
        }

        if succ_flag && let Err(e) = tag_file(filepath, img.image.tags.unwrap()) {
            eprintln!("While tagging {}: {}", line, e);
        }

        line.clear();
        path_buf.clear();
    }
    Ok(())
}
