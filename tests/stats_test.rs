extern crate pihole_api;

use std::collections::HashMap;

#[test]
fn test_summary() {
    let mut data = HashMap::new();
    let input: &[u8] = &[
        0xd2, 0xff, 0xff, 0xff, 0xff, 0xd2, 0x00, 0x00, 0x00, 0x07, 0xd2, 0x00, 0x00, 0x00, 0x02,
        0xca, 0x41, 0xe4, 0x92, 0x49, 0xd2, 0x00, 0x00, 0x00, 0x06, 0xd2, 0x00, 0x00, 0x00, 0x03,
        0xd2, 0x00, 0x00, 0x00, 0x02, 0xd2, 0x00, 0x00, 0x00, 0x03, 0xd2, 0x00, 0x00, 0x00, 0x03,
        0xcc, 0x02, 0xc1
    ];
    data.insert("stats".to_owned(), input);

    let client = pihole_api::test(data);
    let mut response = client.get("/admin/api/stats/summary").dispatch();
    let body = response.body_string();

    assert!(body.is_some());
    assert_eq!(
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
        }",
        body.unwrap()
    );
}