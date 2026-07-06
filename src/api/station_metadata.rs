// Shortwave - station_metadata.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::fmt::Write;
use std::str::FromStr;

use gtk::glib;
use inflector::Inflector;
use serde::{Deserialize, Deserializer, Serializer};
use url::Url;

#[derive(glib::Boxed, Default, Debug, Clone, Serialize, Deserialize)]
#[boxed_type(name = "SwStationMetadata")]
pub struct StationMetadata {
    pub changeuuid: String,
    pub stationuuid: String,
    pub name: String,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub url: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub url_resolved: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub homepage: Option<Url>,
    #[serde(serialize_with = "url_to_str")]
    #[serde(deserialize_with = "str_to_url")]
    pub favicon: Option<Url>,
    pub tags: String,
    pub country: String,
    pub countrycode: String,
    pub state: String,
    pub language: String,
    pub languagecodes: String,
    pub votes: i32,
    pub lastchangetime: Option<String>,
    pub lastchangetime_iso8601: Option<String>,
    pub codec: String,
    pub bitrate: i32,
    pub hls: i32,
    pub lastcheckok: i32,
    pub lastchecktime: Option<String>,
    pub lastchecktime_iso8601: Option<String>,
    pub lastcheckoktime: Option<String>,
    pub lastcheckoktime_iso8601: Option<String>,
    pub lastlocalchecktime: Option<String>,
    pub clicktimestamp: Option<String>,
    pub clicktimestamp_iso8601: Option<String>,
    pub clickcount: i32,
    pub clicktrend: i32,
    pub ssl_error: i32,
    pub geo_lat: Option<f32>,
    pub geo_long: Option<f32>,
    pub has_extended_info: bool,
}

impl StationMetadata {
    pub fn formatted_tags(&self) -> String {
        let tags = self.tags.split(',');
        let mut formatted = String::new();
        for tag in tags {
            write!(formatted, ", {}", tag.to_title_case()).unwrap_or_default();
        }
        formatted.split_at(2).1.to_string()
    }
}

impl StationMetadata {
    /// Create metadata for a new local station.
    pub fn new(name: String, url: Url) -> Self {
        Self {
            name,
            url: Some(url),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metadata() -> StationMetadata {
        StationMetadata {
            changeuuid: "chg-1".to_string(),
            stationuuid: "stu-1".to_string(),
            name: "Test FM".to_string(),
            url: Some(Url::parse("https://example.com/stream").unwrap()),
            url_resolved: Some(Url::parse("https://example.com/stream-resolved").unwrap()),
            homepage: Some(Url::parse("https://example.com").unwrap()),
            favicon: Some(Url::parse("https://example.com/icon.png").unwrap()),
            tags: "rock,pop,jazz".to_string(),
            country: "Germany".to_string(),
            countrycode: "DE".to_string(),
            state: "Bavaria".to_string(),
            language: "german".to_string(),
            languagecodes: "de".to_string(),
            votes: 42,
            lastchangetime: Some("2024-01-01 00:00:00".to_string()),
            lastchangetime_iso8601: None,
            codec: "MP3".to_string(),
            bitrate: 128,
            hls: 0,
            lastcheckok: 1,
            lastchecktime: None,
            lastchecktime_iso8601: None,
            lastcheckoktime: None,
            lastcheckoktime_iso8601: None,
            lastlocalchecktime: None,
            clicktimestamp: None,
            clicktimestamp_iso8601: None,
            clickcount: 10,
            clicktrend: 1,
            ssl_error: 0,
            geo_lat: Some(48.13),
            geo_long: Some(11.57),
            has_extended_info: true,
        }
    }

    #[test]
    fn test_formatted_tags() {
        let metadata = make_metadata();
        let formatted = metadata.formatted_tags();
        // split on comma, title-case each, join with ", "
        assert_eq!(formatted, "Rock, Pop, Jazz");
    }

    #[test]
    fn test_formatted_tags_empty() {
        let mut metadata = make_metadata();
        metadata.tags = String::new();
        let formatted = metadata.formatted_tags();
        assert!(formatted.is_empty(), "Expected empty, got: {formatted:?}");
    }

    #[test]
    fn test_serde_roundtrip() {
        let metadata = make_metadata();
        let serialized = serde_json::to_value(&metadata).unwrap();
        let deserialized: StationMetadata = serde_json::from_value(serialized).unwrap();

        assert_eq!(deserialized.changeuuid, "chg-1");
        assert_eq!(deserialized.stationuuid, "stu-1");
        assert_eq!(deserialized.name, "Test FM");
        assert_eq!(
            deserialized.url,
            Some(Url::parse("https://example.com/stream").unwrap())
        );
        assert_eq!(
            deserialized.homepage,
            Some(Url::parse("https://example.com").unwrap())
        );
        assert_eq!(deserialized.tags, "rock,pop,jazz");
        assert_eq!(deserialized.votes, 42);
        assert_eq!(deserialized.geo_lat, Some(48.13));
    }

    #[test]
    fn test_missing_url_field_deserializes_to_none() {
        let json = serde_json::json!({
            "changeuuid": "x",
            "stationuuid": "y",
            "name": "z",
            "url": "",
            "url_resolved": "",
            "homepage": "",
            "favicon": "",
            "tags": "",
            "country": "",
            "countrycode": "",
            "state": "",
            "language": "",
            "languagecodes": "",
            "votes": 0,
            "lastchangetime": null,
            "lastchangetime_iso8601": null,
            "codec": "",
            "bitrate": 0,
            "hls": 0,
            "lastcheckok": 0,
            "lastchecktime": null,
            "lastchecktime_iso8601": null,
            "lastcheckoktime": null,
            "lastcheckoktime_iso8601": null,
            "lastlocalchecktime": null,
            "clicktimestamp": null,
            "clicktimestamp_iso8601": null,
            "clickcount": 0,
            "clicktrend": 0,
            "ssl_error": 0,
            "geo_lat": null,
            "geo_long": null,
            "has_extended_info": false,
        });
        let deserialized: StationMetadata = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.url, None);
        assert_eq!(deserialized.url_resolved, None);
        assert_eq!(deserialized.homepage, None);
        assert_eq!(deserialized.favicon, None);
    }

    #[test]
    fn test_new_local_station() {
        let url = Url::parse("file:///home/user/music.mp3").unwrap();
        let metadata = StationMetadata::new("My Local Station".to_string(), url.clone());
        assert_eq!(metadata.name, "My Local Station");
        assert_eq!(metadata.url, Some(url));
    }
}

fn url_to_str<S>(url: &Option<Url>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value = if let Some(url) = url {
        url.as_str()
    } else {
        ""
    };
    serializer.serialize_str(value)
}

fn str_to_url<'de, D>(deserializer: D) -> Result<Option<Url>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Url::from_str(&s).ok())
}
