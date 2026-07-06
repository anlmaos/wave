// Shortwave - library_state.rs
// Copyright (C) 2024  Felix Häcker <haeckerfelix@gnome.org>
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

use gtk::glib;
use gtk::glib::Enum;

#[derive(Display, Copy, Debug, Clone, EnumString, Eq, PartialEq, Enum)]
#[repr(u32)]
#[enum_type(name = "SwLibraryStatus")]
#[derive(Default)]
pub enum SwLibraryStatus {
    #[default]
    Content,
    Empty,
    Offline,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display() {
        assert_eq!(SwLibraryStatus::Content.to_string(), "Content");
        assert_eq!(SwLibraryStatus::Empty.to_string(), "Empty");
        assert_eq!(SwLibraryStatus::Offline.to_string(), "Offline");
    }

    #[test]
    fn test_default_is_content() {
        assert_eq!(SwLibraryStatus::default(), SwLibraryStatus::Content);
    }

    #[test]
    fn test_from_str_ok() {
        assert_eq!(
            SwLibraryStatus::from_str("Content").unwrap(),
            SwLibraryStatus::Content
        );
        assert_eq!(
            SwLibraryStatus::from_str("Empty").unwrap(),
            SwLibraryStatus::Empty
        );
        assert_eq!(
            SwLibraryStatus::from_str("Offline").unwrap(),
            SwLibraryStatus::Offline
        );
    }

    #[test]
    fn test_from_str_rejects_unknown() {
        assert!(SwLibraryStatus::from_str("Bogus").is_err());
    }

    #[test]
    fn test_roundtrip() {
        for variant in [
            SwLibraryStatus::Content,
            SwLibraryStatus::Empty,
            SwLibraryStatus::Offline,
        ] {
            let serialized = variant.to_string();
            let parsed = SwLibraryStatus::from_str(&serialized).unwrap();
            assert_eq!(parsed, variant);
        }
    }
}
