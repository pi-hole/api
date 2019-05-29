// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Gravity Database Models
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

#[database("gravity_database")]
pub struct GravityDatabase(diesel::SqliteConnection);
