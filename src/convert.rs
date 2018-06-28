/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  Convert booleans returned as strings.
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

/// Convert booleans returned as strings.
pub fn as_bool(t: &str) -> bool {
  match t.to_lowercase().as_str() {
    "true" | "1" => true,
    "false" | "0" => false,
    _ => false
  }
}
