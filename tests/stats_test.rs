extern crate pihole_api;
extern crate rmp;

use std::collections::HashMap;
use rmp::encode;

/// Test an API endpoint by inputting test data and checking the response
fn test_endpoint(endpoint: &str, ftl_command: &str, ftl_data: Vec<u8>, expected: &str) {
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

fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}

#[test]
fn test_summary() {
    let mut data = Vec::new();
    encode::write_i32(&mut data, -1).unwrap();
    encode::write_i32(&mut data, 7).unwrap();
    encode::write_i32(&mut data, 2).unwrap();
    encode::write_f32(&mut data, 28.571428298950197).unwrap();
    encode::write_i32(&mut data, 6).unwrap();
    encode::write_i32(&mut data, 3).unwrap();
    encode::write_i32(&mut data, 2).unwrap();
    encode::write_i32(&mut data, 3).unwrap();
    encode::write_i32(&mut data, 3).unwrap();
    encode::write_u8(&mut data, 2).unwrap();
    write_eom(&mut data);

    test_endpoint(
        "/admin/api/stats/summary",
        "stats",
        data,
        "{\
            \"data\":{\
                \"blocked_queries\":2,\
                \"cached_queries\":2,\
                \"domains_blocked\":-1,\
                \"forwarded_queries\":3,\
                \"percent_blocked\":28.571428298950197,\
                \"status\":2,\
                \"total_clients\":3,\
                \"total_queries\":7,\
                \"unique_clients\":3,\
                \"unique_domains\":6\
            },\
            \"errors\":[]\
        }"
    );
}

#[test]
fn test_top_domains() {
    let mut data = Vec::new();
    encode::write_i32(&mut data, 10).unwrap();
    encode::write_str(&mut data, "example.com").unwrap();
    encode::write_i32(&mut data, 7).unwrap();
    encode::write_str(&mut data, "example.net").unwrap();
    encode::write_i32(&mut data, 3).unwrap();
    write_eom(&mut data);

    test_endpoint(
        "/admin/api/stats/top_domains",
        "top-domains (10)  ",
        data,
        "{\
            \"data\":{\
                \"top_domains\":{\
                    \"example.com\":7,\
                    \"example.net\":3\
                },\
                \"total_queries\":10\
            },\
            \"errors\":[]\
        }"
    );
}

#[test]
fn test_top_domains_limit() {
    let mut data = Vec::new();
    encode::write_i32(&mut data, 10).unwrap();
    encode::write_str(&mut data, "example.com").unwrap();
    encode::write_i32(&mut data, 7).unwrap();
    write_eom(&mut data);

    test_endpoint(
        "/admin/api/stats/top_domains?limit=1",
        "top-domains (1)  ",
        data,
        "{\
            \"data\":{\
                \"top_domains\":{\
                    \"example.com\":7\
                },\
                \"total_queries\":10\
            },\
            \"errors\":[]\
        }"
    );
}
