#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Ethereum [token list](https://tokenlists.org/) standard

use std::collections::HashMap;

use chrono::{DateTime, FixedOffset};
use semver::Version;
use serde::{Deserialize, Serialize};
use url::Url;

/// A list of Ethereum token metadata conforming to the [token list schema].
///
/// [token list schema]: https://uniswap.org/tokenlist.schema.json
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenList {
    /// The name of the token list
    pub name: String,

    /// The timestamp of this list version; i.e. when this immutable version of
    /// the list was created
    pub timestamp: DateTime<FixedOffset>,

    /// The version of the list, used in change detection
    #[serde(with = "version")]
    pub version: Version,

    /// A URI for the logo of the token list; prefer SVG or PNG of size 256x256
    #[serde(rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<Url>,

    /// Keywords associated with the contents of the list; may be used in list
    /// discoverability
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<String>,

    /// A mapping of tag identifiers to their name and description
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub tags: HashMap<String, Tag>,

    /// The list of tokens included in the list
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tokens: Vec<Token>,
}

/// Metadata for a single token in a token list
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// The name of the token
    pub name: String,

    //
    /// The symbol for the token; must be alphanumeric.
    pub symbol: String,

    /// The checksummed address of the token on the specified chain ID
    pub address: String,

    /// The chain ID of the Ethereum network where this token is deployed
    pub chain_id: u16,

    /// The number of decimals for the token balance
    pub decimals: u16,

    /// A URI to the token logo asset; if not set, interface will attempt to
    /// find a logo based on the token address; suggest SVG or PNG of size 64x64
    #[serde(rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<Url>,

    /// An array of tag identifiers associated with the token; tags are defined
    /// at the list level
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// An object containing any arbitrary or vendor-specific token metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extensions: HashMap<String, Option<ExtensionValue>>,
}

/// Definition of a tag that can be associated with a token via its identifier
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// The name of the tag
    pub name: String,

    /// A user-friendly description of the tag
    pub description: String,
}

/// The value for a user-defined extension.
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum ExtensionValue {
    String(String),
    Number(Number),
    Boolean(bool),
}

/// A number
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

mod version {
    use semver::Version;
    use serde::{de, ser::SerializeStruct, Deserialize};

    pub fn serialize<S>(value: &Version, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut version = serializer.serialize_struct("Version", 3)?;
        version.serialize_field("major", &value.major)?;
        version.serialize_field("minor", &value.minor)?;
        version.serialize_field("patch", &value.patch)?;
        version.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Version, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InternalVersion {
            major: u64,
            minor: u64,
            patch: u64,
        }

        InternalVersion::deserialize(deserializer).map(|v| Version::new(v.major, v.minor, v.patch))
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use serde_json::json;

    use super::*;

    #[test]
    fn can_serialize_deserialize_required_fields() {
        let data_json = json!({
            "name": "TELcoins",
            "timestamp": "2021-07-05T20:25:22+00:00",
            "version": { "major": 0, "minor": 1, "patch": 0 },
            "tokens": [
                {
                    "name": "Telcoin",
                    "symbol": "TEL",
                    "address": "0x467bccd9d29f223bce8043b84e8c8b282827790f",
                    "chainId": 1,
                    "decimals": 2
                }
            ]
        });

        let data_rs = TokenList {
            name: "TELcoins".to_owned(),
            timestamp: FixedOffset::west(0).ymd(2021, 7, 5).and_hms(20, 25, 22),
            version: Version::new(0, 1, 0),
            logo_uri: None,
            keywords: vec![],
            tags: HashMap::new(),
            tokens: vec![Token {
                name: "Telcoin".to_owned(),
                symbol: "TEL".to_owned(),
                address: "0x467bccd9d29f223bce8043b84e8c8b282827790f".to_owned(),
                chain_id: 1,
                decimals: 2,
                logo_uri: None,
                tags: vec![],
                extensions: HashMap::new(),
            }],
        };

        assert_eq!(serde_json::to_value(&data_rs).unwrap(), data_json);

        let token_list: TokenList = serde_json::from_value(data_json).unwrap();

        assert_eq!(token_list, data_rs);
    }

    #[test]
    fn can_serialize_deserialize_all_fields() {
        let data_json = json!({
            "name": "TELcoins",
            "timestamp": "2021-07-05T20:25:22+00:00",
            "version": { "major": 0, "minor": 1, "patch": 0 },
            "logoURI": "https://raw.githubusercontent.com/telcoin/token-lists/master/assets/logo-telcoin-250x250.png",
            "keywords": ["defi", "telcoin"],
            "tags": {
                "telcoin": {
                    "description": "Part of the Telcoin ecosystem.",
                    "name": "telcoin"
                }
            },
            "tokens": [
                {
                    "name": "Telcoin",
                    "symbol": "TEL",
                    "address": "0x467bccd9d29f223bce8043b84e8c8b282827790f",
                    "chainId": 1,
                    "decimals": 2,
                    "logoURI": "https://raw.githubusercontent.com/telcoin/token-lists/master/assets/logo-telcoin-250x250.png",
                    "tags": ["telcoin"],
                    "extensions": {
                        "is_mapped_to_matic": true,
                        "matic_address": "0xdf7837de1f2fa4631d716cf2502f8b230f1dcc32",
                        "matic_chain_id": 137
                    }
                }
            ]
        });

        let logo_uri: Url = "https://raw.githubusercontent.com/telcoin/token-lists/master/assets/logo-telcoin-250x250.png".parse().unwrap();
        let data_rs = TokenList {
            name: "TELcoins".to_owned(),
            timestamp: FixedOffset::west(0).ymd(2021, 7, 5).and_hms(20, 25, 22),
            version: Version::new(0, 1, 0),
            logo_uri: Some(logo_uri.clone()),
            keywords: vec!["defi".to_owned(), "telcoin".to_owned()],
            tags: vec![(
                "telcoin".to_owned(),
                Tag {
                    name: "telcoin".to_owned(),
                    description: "Part of the Telcoin ecosystem.".to_owned(),
                },
            )]
            .into_iter()
            .collect(),
            tokens: vec![Token {
                name: "Telcoin".to_owned(),
                symbol: "TEL".to_owned(),
                address: "0x467bccd9d29f223bce8043b84e8c8b282827790f".to_owned(),
                chain_id: 1,
                decimals: 2,
                logo_uri: Some(logo_uri),
                tags: vec!["telcoin".to_owned()],
                extensions: vec![
                    (
                        "is_mapped_to_matic".to_owned(),
                        Some(ExtensionValue::Boolean(true)),
                    ),
                    (
                        "matic_address".to_owned(),
                        Some(ExtensionValue::String(
                            "0xdf7837de1f2fa4631d716cf2502f8b230f1dcc32".to_owned(),
                        )),
                    ),
                    (
                        "matic_chain_id".to_owned(),
                        Some(ExtensionValue::Number(Number::Integer(137))),
                    ),
                ]
                .into_iter()
                .collect(),
            }],
        };

        assert_eq!(serde_json::to_value(&data_rs).unwrap(), data_json,);

        let token_list: TokenList = serde_json::from_value(data_json).unwrap();

        assert_eq!(token_list, data_rs);
    }
}
