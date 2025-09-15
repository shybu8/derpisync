use serde::Deserialize;
use std::env;
use std::process::{Command, Stdio, exit};

#[derive(Debug, Deserialize)]
struct ImagesEndpoint {
    pub image: Image,
}

#[derive(Debug, Deserialize)]
struct ImagesEndpointDuplicate {
    pub image: ImageDuplicate,
}

#[derive(Debug, Deserialize)]
struct Image {
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ImageDuplicate {
    pub duplicate_of: u64,
}

const ERR_CHILD_EXEC: &str = "ERROR: Unknown error during child execution";

fn query(id: u64) -> String {
    let query = format!("https://derpibooru.org/api/v1/json/images/{}", id);
    reqwest::blocking::get(query)
        .expect("ERROR: Response receiveal error")
        .text()
        .expect("ERROR: Getting responce text error")
}

fn main() {
    let mut args = env::args();
    if args.len() != 2 {
        eprintln!("ERROR: Wrong number of arguments (should be only one image id)");
        exit(1);
    }
    let _ = args.next(); // Program name
    let file_name = args.next().unwrap();
    let image_id = file_name
        .split(".")
        .next()
        .expect("ERROR: Invalid file name");
    let image_id = u64::from_str_radix(&image_id, 10).expect("ERROR: Invalid image id");

    let tmsu_ver_status = Command::new("tmsu")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .expect(ERR_CHILD_EXEC);
    if !tmsu_ver_status.success() {
        eprintln!("ERROR: You are likely doesn't have tmsu installed");
        exit(1);
    }

    let tmsu_info_status = Command::new("tmsu")
        .arg("info")
        .stdout(Stdio::null())
        .status()
        .expect(ERR_CHILD_EXEC);
    if !tmsu_info_status.success() {
        eprintln!("ERROR: There is no tmsu database");
        exit(1);
    }

    let json = query(image_id);
    let img = match serde_json::from_str::<ImagesEndpoint>(&json) {
        Ok(r) => r,
        Err(_) => {
            let duplicate = serde_json::from_str::<ImagesEndpointDuplicate>(&json)
                .expect("ERROR: Invalid json");
            let json2 = query(duplicate.image.duplicate_of);
            serde_json::from_str::<ImagesEndpoint>(&json2).expect("ERROR: Invalid json")
        }
    };

    let mut tmsu_tag = Command::new("tmsu");
    tmsu_tag.stdout(Stdio::null());
    tmsu_tag.arg("tag");
    tmsu_tag.arg(file_name);
    for a in img.image.tags {
        tmsu_tag.arg(a);
    }
    let tmsu_tag_status = tmsu_tag.status().expect(ERR_CHILD_EXEC);
    if !tmsu_tag_status.success() {
        eprintln!("ERROR: Tagging process was unsuccessful");
        exit(1);
    }
}
