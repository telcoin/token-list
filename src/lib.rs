#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Ethereum [token list](https://tokenlists.org/) standard
//!
//! # Examples
//!
//! ```no_run
//! use token_list::TokenList;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // requires enabling the `from-uri` feature
//!     let token_list = TokenList::from_uri("https://defi.cmc.eth.link").await?;
//!     
//!     assert_eq!(token_list.name, "CMC DeFi");
//!     
//!     Ok(())
//! }
//! ```

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

impl TokenList {
    /// Constructs a [`TokenList`] from the JSON contents of the specified URI.
    ///
    /// **Note**: This must be called from a running tokio >1.0.0 runtime.
    #[cfg(feature = "from-uri")]
    pub async fn from_uri<T: reqwest::IntoUrl>(uri: T) -> Result<Self, Error> {
        Ok(reqwest::get(uri).await?.error_for_status()?.json().await?)
    }

    /// Constructs a [`TokenList`] from the JSON contents of the specified URI.
    ///
    /// **Note**: This must be called from a running tokio 0.1.x runtime.
    #[cfg(feature = "from-uri-compat")]
    pub async fn from_uri_compat<T: reqwest09::IntoUrl>(uri: T) -> Result<Self, Error> {
        use futures::compat::Future01CompatExt;
        use futures01::Future;
        use reqwest09::r#async::{Client, Response};

        let fut = Client::new()
            .get(uri)
            .send()
            .and_then(Response::error_for_status)
            .and_then(|mut res| res.json())
            .compat();

        Ok(fut.await?)
    }
}

/// Metadata for a single token in a token list
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// The name of the token
    pub name: String,

    /// The symbol for the token; must be alphanumeric.
    pub symbol: String,

    /// The checksummed address of the token on the specified chain ID
    pub address: String,

    /// The chain ID of the Ethereum network where this token is deployed
    pub chain_id: u32,

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

impl Token {
    /// Gets the value of `polygonAddress` if present (and a `String`) in the
    /// `extensions` map.
    pub fn polygon_address(&self) -> Option<&str> {
        self.extensions
            .get("polygonAddress")
            .and_then(|val| val.as_ref().and_then(|v| v.as_str()))
    }
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

impl ExtensionValue {
    /// If the `ExtensionValue` is a `String`, returns the associated `str`.
    /// Returns `None` otherwise.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ExtensionValue::String(val) => Some(val),
            ExtensionValue::Number(_) => None,
            ExtensionValue::Boolean(_) => None,
        }
    }

    /// If the `ExtensionValue` is a `Boolean`, returns the associated `bool`.
    /// Returns `None` otherwise.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ExtensionValue::String(_) => None,
            ExtensionValue::Number(_) => None,
            ExtensionValue::Boolean(val) => Some(*val),
        }
    }

    /// If the `ExtensionValue` is a `Number` and an `i64`, returns the
    /// associated `i64`. Returns `None` otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            ExtensionValue::String(_) => None,
            ExtensionValue::Number(val) => val.as_i64(),
            ExtensionValue::Boolean(_) => None,
        }
    }

    /// If the `ExtensionValue` is a `Number` and an `f64`, returns the
    /// associated `f64`. Returns `None` otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ExtensionValue::String(_) => None,
            ExtensionValue::Number(val) => val.as_f64(),
            ExtensionValue::Boolean(_) => None,
        }
    }
}

/// A number
#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
#[serde(untagged)]
#[allow(missing_docs)]
pub enum Number {
    Integer(i64),
    Float(f64),
}

impl Number {
    /// If the `Number` is a `i64`, returns the associated `i64`. Returns `None`
    /// otherwise.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Number::Integer(val) => Some(*val),
            Number::Float(_) => None,
        }
    }

    /// If the `Number` is a `f64`, returns the associated `f64`. Returns `None`
    /// otherwise.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Number::Integer(_) => None,
            Number::Float(val) => Some(*val),
        }
    }
}

/// Represents all errors that can occur when using this library.
#[cfg(any(feature = "from-uri", feature = "from-uri-compat"))]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// HTTP/TCP etc. transport level error.
    #[cfg(feature = "from-uri")]
    #[error(transparent)]
    Transport(#[from] reqwest::Error),

    /// HTTP/TCP etc. transport level error.
    #[cfg(feature = "from-uri-compat")]
    #[error(transparent)]
    TransportCompat(#[from] reqwest09::Error),
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

    const TELCOINS_TOKEN_LIST_URI: &str =
        "https://raw.githubusercontent.com/telcoin/token-lists/e6a4cd7/telcoins.json";

    #[cfg(feature = "from-uri")]
    #[tokio::test]
    async fn from_uri() {
        let token_list = TokenList::from_uri(TELCOINS_TOKEN_LIST_URI).await.unwrap();
        dbg!(&token_list);
    }

    #[cfg(feature = "from-uri-compat")]
    #[test]
    fn from_uri_compat() {
        use futures::future::{FutureExt, TryFutureExt};
        use tokio01::runtime::Runtime;

        let mut rt = Runtime::new().unwrap();

        rt.block_on(
            TokenList::from_uri_compat(TELCOINS_TOKEN_LIST_URI)
                .boxed()
                .compat(),
        )
        .unwrap();
    }

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
                        "is_mapped_to_polygon": true,
                        "polygon_address": "0xdf7837de1f2fa4631d716cf2502f8b230f1dcc32",
                        "polygon_chain_id": 137
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
                        "is_mapped_to_polygon".to_owned(),
                        Some(ExtensionValue::Boolean(true)),
                    ),
                    (
                        "polygon_address".to_owned(),
                        Some(ExtensionValue::String(
                            "0xdf7837de1f2fa4631d716cf2502f8b230f1dcc32".to_owned(),
                        )),
                    ),
                    (
                        "polygon_chain_id".to_owned(),
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
