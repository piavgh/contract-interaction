use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    title: String,
}

// implement trait Display for Metadata
impl std::fmt::Display for Metadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = format!("Title: {}\n", self.title);

        write!(f, "{}", title)
    }
}

pub fn parse_metadata(metadata: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let json_str = hex_to_string(metadata)?;

    let metadata = parse_metadata_from_json(&json_str)?;

    Ok(metadata)
}

fn hex_to_string(hex: &str) -> Result<String, hex::FromHexError> {
    // Remove "0x" prefix and convert hex string to json string
    hex::decode(hex.trim_start_matches("0x"))
        .map(|bytes| String::from_utf8(bytes).expect("Invalid UTF-8"))
}

fn parse_metadata_from_json(json_str: &str) -> Result<Metadata, serde_json::Error> {
    let metadata: Metadata = serde_json::from_str(json_str)?;
    Ok(metadata)
}
