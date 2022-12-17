use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};

#[derive(Serialize, Deserialize, Debug)]
struct Version {
    resource: String,
    client: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Details {
    version: Version,
}

impl Details {
    #[must_use]
    pub fn get(path: Option<&str>) -> Self {
        let path = path.unwrap_or("details.json");
        let file = File::open(path).expect("Failed to open details file");
        serde_json::from_reader(BufReader::new(file)).expect("Failed to deserialize details")
    }

    pub fn save(self, path: Option<&str>) {
        let path = path.unwrap_or("details.json");
        let file = File::create(path).expect("Failed to open details file");
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
        Details::get(Some(generate_path().as_str()));
    }

    #[test]
    #[should_panic]
    #[allow(unused_must_use)]
    fn panic_on_invalid_deserializing() {
        let path = generate_path();
        let res = catch_unwind(|| {
            if let Ok(file) = File::create(&path) {
                if serde_json::to_writer(file, &json!("{}")).is_ok() {
                    Details::get(Some(path.as_str()));
                }
            }
        });
        std::fs::remove_file(path);
        res.unwrap();
    }
}
