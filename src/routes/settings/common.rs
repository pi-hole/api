// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Functions For Settings Endpoints
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

/// Convert booleans returned as strings.
pub fn as_bool(boolean_string: &str) -> bool {
    match boolean_string.to_lowercase().as_str() {
        "true" | "1" => true,
        "false" | "0" => false,
        _ => false
    }
}

#[cfg(test)]
mod tests {
    use super::as_bool;

    #[test]
    fn test_as_bool() {
        assert_eq!(as_bool("FALSE"), false);
        assert_eq!(as_bool("false"), false);
        assert_eq!(as_bool("TRUE"), true);
        assert_eq!(as_bool("tRuE"), true);
        assert_eq!(as_bool("1"), true);
        assert_eq!(as_bool("0"), false);
    }
}
