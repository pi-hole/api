extern crate pihole_api;

use std::collections::HashMap;

/// Test an API endpoint by inputting test data and checking the response
pub fn test_endpoint(endpoint: &str, ftl_command: &str, ftl_data: Vec<u8>, expected: &str) {
    // Add the test data
    let mut data = HashMap::new();
    data.insert(ftl_command.to_owned(), ftl_data);

    // Start the test client
    let client = pihole_api::test(data);

    // Get the response
    let mut response = client.get(endpoint).dispatch();
    let body = response.body_string();

    // Check against expected output
    assert!(body.is_some());
    assert_eq!(expected, body.unwrap());
}

pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}
