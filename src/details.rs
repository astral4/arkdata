use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize)]
pub struct Version {
    #[serde(rename = "resVersion")]
    resource: String,
    #[serde(rename = "clientVersion")]
    client: String,
}

#[derive(Serialize, Deserialize)]
pub struct Details {
    #[serde(flatten)]
    pub version: Version,
    path: String,
}

impl Details {
    #[must_use]
    pub fn get(path: &str) -> Self {
        let file = File::open(path).expect("Failed to open details file");
        let mut details: Details =
            serde_json::from_reader(BufReader::new(file)).expect("Failed to deserialize details");
        details.path = path.to_string();
        details
    }

    pub fn save(self) {
        let file = File::create(&self.path).expect("Failed to open details file");
        serde_json::to_writer(file, &self).expect("Failed to serialize details");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::panic::catch_unwind;
    use uuid::Uuid;

    fn generate_path() -> String {
        format!("{}{}", Uuid::new_v4(), ".json")
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_nonexistent_file() {
        Details::get(generate_path().as_str());
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_invalid_deserializing() {
        let path = generate_path();
        let res = catch_unwind(|| {
            if let Ok(file) = File::create(&path) {
                if serde_json::to_writer(file, &json!("{}")).is_ok() {
                    Details::get(path.as_str());
                }
            }
        });
        std::fs::remove_file(path);
        res.unwrap();
    }
}
