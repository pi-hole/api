/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Common Test Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

extern crate tempfile;

use config::PiholeFile;
use rocket::http::Method;
use serde_json;
use setup;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

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

/// Test an endpoint with multiple mocked files
///
/// The `files` argument is a `Vec` of `(file, initial, expected)`
/// where `file` is the `PiholeFile` to mock, `initial` is the initial content of the file,
/// and `expected` is the expected content of the file after the test.
pub fn test_endpoint_config_multi(
    method: Method,
    endpoint: &str,
    files: Vec<(PiholeFile, &str, &str)>,
    expected: serde_json::Value
) {
    let mut data = HashMap::new();
    let mut file_data = Vec::new();

    // Create temporary mock files
    for (pihole_file, initial, expected) in files {
        // Create the file handle
        let mut file = tempfile::tempfile().unwrap();

        // Write the initial data to the file
        write!(file, "{}", initial).unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();

        // Save the file for the test and verification
        file_data.push((file.try_clone().unwrap(), expected));
        data.insert(pihole_file, file);
    }

    // Test the endpoint
    test_endpoint(method, endpoint, HashMap::new(), data, expected);

    // Verify files are as expected at the end
    let mut buffer = String::new();
    for (mut file, expected) in file_data {
        file.seek(SeekFrom::Start(0)).unwrap();
        file.read_to_string(&mut buffer).unwrap();

        assert_eq!(buffer, expected);
        buffer.clear();
    }
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
