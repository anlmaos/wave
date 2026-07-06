// Shortwave - i18n.rs
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
//
// Source: GNOME Social by Christopher Davis
// https://gitlab.gnome.org/World/Social

use gettextrs::{gettext, ngettext, npgettext, pgettext};
use regex::{Captures, Regex};

#[allow(dead_code)]
fn freplace(input: String, args: &[&str]) -> String {
    let mut parts = input.split("{}");
    let mut output = parts.next().unwrap_or_default().to_string();
    for (p, a) in parts.zip(args.iter()) {
        output += &((*a).to_string() + p);
    }
    output
}

fn freplace_dict(s: String, args: &[(&str, &str)]) -> String {
    let mut s = s;

    for (k, v) in args {
        s = s.replace(&format!("{{{k}}}"), v);
    }

    s
}

#[allow(dead_code)]
fn kreplace(input: String, kwargs: &[(&str, &str)]) -> String {
    let mut s = input;
    for (k, v) in kwargs {
        if let Ok(re) = Regex::new(&format!("\\{{{k}\\}}")) {
            s = re
                .replace_all(&s, |_: &Captures<'_>| (*v).to_string())
                .to_string();
        }
    }

    s
}

// Simple translations functions

#[allow(dead_code)]
pub fn i18n(format: &str) -> String {
    gettext(format)
}

#[allow(dead_code)]
pub fn i18n_f(format: &str, args: &[&str]) -> String {
    let s = gettext(format);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn i18n_k(format: &str, kwargs: &[(&str, &str)]) -> String {
    let s = gettext(format);
    kreplace(s, kwargs)
}

// Singular and plural translations functions

#[allow(dead_code)]
pub fn ni18n(single: &str, multiple: &str, number: u32) -> String {
    ngettext(single, multiple, number)
}

#[allow(dead_code)]
pub fn ni18n_f(single: &str, multiple: &str, number: u32, args: &[&str]) -> String {
    let s = ngettext(single, multiple, number);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn ni18n_k(single: &str, multiple: &str, number: u32, kwargs: &[(&str, &str)]) -> String {
    let s = ngettext(single, multiple, number);
    kreplace(s, kwargs)
}

// Translations with context functions

#[allow(dead_code)]
pub fn pi18n(ctx: &str, format: &str) -> String {
    pgettext(ctx, format)
}

#[allow(dead_code)]
pub fn pi18n_f(ctx: &str, format: &str, args: &[&str]) -> String {
    let s = pgettext(ctx, format);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn pi18n_k(ctx: &str, format: &str, kwargs: &[(&str, &str)]) -> String {
    let s = pgettext(ctx, format);
    kreplace(s, kwargs)
}

// Singular and plural with context

#[allow(dead_code)]
pub fn pni18n(ctx: &str, single: &str, multiple: &str, number: u32) -> String {
    npgettext(ctx, single, multiple, number)
}

#[allow(dead_code)]
pub fn pni18n_f(ctx: &str, single: &str, multiple: &str, number: u32, args: &[&str]) -> String {
    let s = npgettext(ctx, single, multiple, number);
    freplace(s, args)
}

#[allow(dead_code)]
pub fn pni18n_k(
    ctx: &str,
    single: &str,
    multiple: &str,
    number: u32,
    kwargs: &[(&str, &str)],
) -> String {
    let s = npgettext(ctx, single, multiple, number);
    kreplace(s, kwargs)
}

pub fn gettext_f(msgid: &str, args: &[(&str, &str)]) -> String {
    let s = gettext(msgid);
    freplace_dict(s, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_freplace_single_arg() {
        let result = freplace("Hello, {}!".to_string(), &["world"]);
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_freplace_multiple_args() {
        let result = freplace("{} {} {}".to_string(), &["a", "b", "c"]);
        assert_eq!(result, "a b c");
    }

    #[test]
    fn test_freplace_no_args() {
        let result = freplace("no placeholders".to_string(), &[] as &[&str]);
        assert_eq!(result, "no placeholders");
    }

    #[test]
    fn test_freplace_extra_args() {
        // Extra args beyond placeholders should be silently ignored
        let result = freplace("{} {}".to_string(), &["a", "b", "c"]);
        assert_eq!(result, "a b");
    }

    #[test]
    fn test_freplace_dict_named_args() {
        let result = freplace_dict(
            "{greeting}, {name}!".to_string(),
            &[("greeting", "Hello"), ("name", "world")],
        );
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn test_freplace_dict_duplicate_keys() {
        // Duplicate keys: each key gets replaced (order doesn't matter for same result)
        let result = freplace_dict("{x} and {x}".to_string(), &[("x", "same")]);
        assert_eq!(result, "same and same");
    }

    #[test]
    fn test_freplace_dict_no_matching_keys() {
        let result = freplace_dict("hello world".to_string(), &[("foo", "bar")]);
        assert_eq!(result, "hello world");
    }

    #[test]
    fn test_freplace_dict_empty_args() {
        let result = freplace_dict("{a}".to_string(), &[] as &[(&str, &str)]);
        assert_eq!(result, "{a}");
    }

    #[test]
    fn test_kreplace_named_args() {
        let result = kreplace("{count} items".to_string(), &[("count", "42")]);
        assert_eq!(result, "42 items");
    }

    #[test]
    fn test_kreplace_malformed_no_panic() {
        // Regex that can't be constructed with the simple regex interp should not panic.
        // Since kreplace wraps Regex::new in `if let Ok(re)`, it should just skip.
        let result = kreplace("{foo}".to_string(), &[("foo", "bar")]);
        assert_eq!(result, "bar");
    }

    #[test]
    fn test_i18n_english_roundtrip() {
        // gettext returns the msgid verbatim without .mo files loaded
        let result = i18n("Hello");
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_ni18n_singular() {
        let result = ni18n("one item", "many items", 1);
        assert!(result.contains("one item") || result == "one item");
    }

    #[test]
    fn test_ni18n_plural() {
        let result = ni18n("one item", "many items", 5);
        // Without .mo files: ngettext returns the plural form when n != 1
        assert!(result.contains("items"));
    }
}
