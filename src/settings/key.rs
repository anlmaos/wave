// Shortwave - key.rs
// Copyright (C) 2021-2025  Felix Häcker <haeckerfelix@gnome.org>
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

#[derive(Display, Debug, Clone, EnumString)]
#[strum(serialize_all = "kebab_case")]
pub enum Key {
    // API
    ApiLookupDomain,
    ApiTimeout,
    ApiConnectTimeout,
    ApiRetryCount,

    // Library
    LibrarySorting,
    LibrarySortingType,

    // Playback
    PlaybackLastStation,
    PlaybackPastTracksCount,
    PlaybackVolume,

    // Recording
    RecordingMaximumDuration,
    RecordingMinimumDuration,
    RecordingMode,
    RecordingTrackDirectory,

    // User Interface
    WindowWidth,
    WindowHeight,
    WindowPreviousWidth,
    WindowPreviousHeight,

    BackgroundPlayback,
    Notifications,

    // Buffer
    BufferDuration,

    // AI
    AiBaseUrl,
    #[strum(serialize = "ai-api-key")]
    AiApiSecret,
    AiModel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display_kebab_case() {
        assert_eq!(Key::ApiLookupDomain.to_string(), "api-lookup-domain");
        assert_eq!(Key::WindowWidth.to_string(), "window-width");
        assert_eq!(Key::BufferDuration.to_string(), "buffer-duration");
        assert_eq!(Key::AiBaseUrl.to_string(), "ai-base-url");
        assert_eq!(Key::AiApiSecret.to_string(), "ai-api-key");
        assert_eq!(Key::AiModel.to_string(), "ai-model");
    }

    #[test]
    fn test_from_str_parses_kebab_case() {
        assert_eq!(
            Key::from_str("api-lookup-domain").unwrap().to_string(),
            Key::ApiLookupDomain.to_string(),
        );
        assert_eq!(
            Key::from_str("window-width").unwrap().to_string(),
            Key::WindowWidth.to_string(),
        );
        assert_eq!(
            Key::from_str("playback-volume").unwrap().to_string(),
            Key::PlaybackVolume.to_string(),
        );
    }

    #[test]
    fn test_from_str_rejects_non_kebab_case() {
        // Standard PascalCase identifiers should not parse.
        assert!(Key::from_str("ApiLookupDomain").is_err());
        assert!(Key::from_str("foo").is_err());
    }

    #[test]
    fn test_roundtrip_all_variants() {
        let variant_labels = vec![
            "api-lookup-domain",
            "api-timeout",
            "api-connect-timeout",
            "api-retry-count",
            "library-sorting",
            "library-sorting-type",
            "playback-last-station",
            "playback-past-tracks-count",
            "playback-volume",
            "recording-maximum-duration",
            "recording-minimum-duration",
            "recording-mode",
            "recording-track-directory",
            "window-width",
            "window-height",
            "window-previous-width",
            "window-previous-height",
            "background-playback",
            "notifications",
            "buffer-duration",
            "ai-base-url",
            "ai-api-key",
            "ai-model",
        ];

        for label in variant_labels {
            let parsed = Key::from_str(label).expect("should parse");
            assert_eq!(parsed.to_string(), label, "roundtrip failed for {label}");
        }
    }
}
