use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use std::fs::{self, File};
use serde::Deserialize;
use std::error::Error;
use std::io::Write;

pub fn get_api_key() -> String {

    match fs::read_to_string("./api_key.txt") {
        Ok(key) => return key,
        Err(_) => {
            File::create("api_key.txt").expect("Was unable to create api_key.txt");
            return get_api_key();
        },
    }
}

pub fn save_api_key(api_key: &String) {
    let mut file = File::create("./api_key.txt").expect("Was unable to open ./api_key.txt");
    write!(file, "{}", api_key).expect("Was unable to edit ./api_key.txt");
}

pub fn is_api_key_valid(key: &str) -> bool {
    match send_translation_request(key, "Test") {
        Ok(_) => return true,
        Err(_) => return false
    }
}

fn send_translation_request(auth: &str, word: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let auth_key = auth;

    let mut headers = HeaderMap::new();
    let auth_header_value = format!("DeepL-Auth-Key {}", auth_key);
    let auth_header = HeaderValue::from_str(&auth_header_value)?;

    headers.insert("Authorization", auth_header);
    headers.insert(USER_AGENT, HeaderValue::from_static("ubersetzer/1.0"));

    let params = vec![
        ("text", word),
        ("target_lang", "FR"),
    ];

    let response = client
        .post("https://api-free.deepl.com/v2/translate")
        .headers(headers)
        .form(&params)
        .send()?;

    // Check the response status
    if response.status().is_success() {
        let body = response.text()?;
        Ok(body)
    } else {
        Err("Request failed".into())
    }
}

pub fn get_translation(auth: &str, word: &String) -> String {

    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Translation {
        detected_source_language: String,
        text: String,
    }

    #[derive(Debug, Deserialize)]
    struct Translations {
        translations: Vec<Translation>,
    }

    match send_translation_request(&auth, &word) {
        Ok(response_body) => {

            let translations: Translations = serde_json::from_str(&response_body).expect("Failed to parse JSON");

            if let Some(translation) = translations.translations.first() {
                return translation.text.clone();
            } else {
                println!("No translations found.");
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
    return String::from("An error has occured in get_translation()");
}