/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Common Test Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use setup;
use std::collections::HashMap;

/// Test an API endpoint by inputting test data and checking the response
pub fn test_endpoint(endpoint: &str, ftl_command: &str, ftl_data: Vec<u8>, expected: &str) {
    // Add the test data
    let mut data = HashMap::new();
    data.insert(ftl_command.to_owned(), ftl_data);

    // Start the test client
    let client = setup::test(data);

    // Get the response
    let mut response = client.get(endpoint).dispatch();
    let body = response.body_string();

    // Check against expected output
    assert!(body.is_some());
    assert_eq!(expected, body.unwrap());
}

/// Add the end of message byte to the data
pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}
