extern crate rmp;

mod common;

use rmp::encode;
use common::{test_endpoint, write_eom};

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
        "top-domains (10)",
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
        "top-domains (1)",
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
