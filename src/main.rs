use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ImagesEndpoint {
    pub image: Image,
}

#[derive(Debug, Deserialize)]
struct Image {
    pub tags: Vec<String>,
}

fn main() {
    let text = reqwest::blocking::get("https://derpibooru.org/api/v1/json/images/1")
        .expect("Response receiveal error")
        .text()
        .expect("Getting responce text error");
    let img = serde_json::from_str::<ImagesEndpoint>(&text);
    dbg!(&img);
    println!("Hello, world!");
}
