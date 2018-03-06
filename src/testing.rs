/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Common Test Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use config::PiholeFile;
use rocket::http::Method;
use serde_json;
use setup;
use std::collections::HashMap;
use std::fs::File;

/// Test an endpoint with mocked FTL data
pub fn test_endpoint_ftl(
    endpoint: &str,
    ftl_command: &str,
    ftl_data: Vec<u8>,
    expected: serde_json::Value
) {
    // Add the test data
    let mut data = HashMap::new();
    data.insert(ftl_command.to_owned(), ftl_data);

    test_endpoint(Method::Get, endpoint, data, HashMap::new(), expected)
}

/// Test an endpoint with a mocked file
pub fn test_endpoint_config(
    endpoint: &str,
    pihole_file: PiholeFile,
    file: File,
    expected: serde_json::Value
) {
    // Add the test data
    let mut data = HashMap::new();
    data.insert(pihole_file, file);

    test_endpoint(Method::Get, endpoint, HashMap::new(), data, expected)
}

/// Test an endpoint by inputting test data and checking the response
pub fn test_endpoint(
    method: Method,
    endpoint: &str,
    ftl_data: HashMap<String, Vec<u8>>,
    config_data: HashMap<PiholeFile, File>,
    expected: serde_json::Value
) {
    // Start the test client
    let client = setup::test(ftl_data, config_data);

    // Get the response
    let mut response = client.req(method, endpoint).dispatch();
    let body = response.body_string();

    // Check that something was returned
    assert!(body.is_some());

    // Check that it is correct JSON
    let parsed: serde_json::Value = serde_json::from_str(&body.unwrap()).unwrap();

    // Check that is is the same as the expected JSON
    assert_eq!(expected, parsed);
}

/// Add the end of message byte to the data
pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}
