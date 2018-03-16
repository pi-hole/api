/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Common Test Functions
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

extern crate serde_json;
extern crate tempfile;

use base64;
use config::PiholeFile;
use rocket::http::{Method, ContentType, Header, Status};
use setup;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

/// Represents a mocked file along with the initial and expected data
struct TestFile {
    pihole_file: PiholeFile,
    temp_file: Option<File>,
    initial_data: String,
    expected_data: String
}

impl TestFile {
    fn new(pihole_file: PiholeFile, initial_data: String, expected_data: String) -> TestFile {
        TestFile {
            pihole_file,
            temp_file: None,
            initial_data,
            expected_data
        }
    }
}

/// Represents a test configuration, with all the data needed to carry out the test
pub struct TestBuilder {
    endpoint: String,
    method: Method,
    headers: Vec<Header<'static>>,
    should_auth: bool,
    body_data: Option<serde_json::Value>,
    ftl_data: HashMap<String, Vec<u8>>,
    test_files: Vec<TestFile>,
    expected_json: serde_json::Value,
    expected_status: Status
}

impl TestBuilder {
    pub fn new() -> TestBuilder {
        TestBuilder {
            endpoint: "".to_owned(),
            method: Method::Get,
            headers: Vec::new(),
            should_auth: true,
            body_data: None,
            ftl_data: HashMap::new(),
            test_files: Vec::new(),
            expected_json: json!({
                "data": [],
                "errors": []
            }),
            expected_status: Status::Ok
        }
    }

    pub fn endpoint(mut self, endpoint: &str) -> Self {
        self.endpoint = endpoint.to_owned();
        self
    }

    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    #[allow(unused)]
    pub fn header<H: Into<Header<'static>>>(mut self, header: H) -> Self {
        self.headers.push(header.into());
        self
    }

    pub fn should_auth(mut self, should_auth: bool) -> Self {
        self.should_auth = should_auth;
        self
    }

    pub fn body(mut self, body: serde_json::Value) -> Self {
        self.body_data = Some(body);
        self
    }

    pub fn ftl(mut self, command: &str, data: Vec<u8>) -> Self {
        self.ftl_data.insert(command.to_owned(), data);
        self
    }

    pub fn file(self, pihole_file: PiholeFile, initial_data: &str) -> Self {
        self.file_expect(pihole_file, initial_data, initial_data)
    }

    pub fn file_expect(
        mut self,
        pihole_file: PiholeFile,
        initial_data: &str,
        expected_data: &str
    ) -> Self {
        let test_file = TestFile::new(
            pihole_file,
            initial_data.to_owned(),
            expected_data.to_owned()
        );
        self.test_files.push(test_file);
        self
    }

    pub fn expect_json(mut self, expected_json: serde_json::Value) -> Self {
        self.expected_json = expected_json;
        self
    }

    pub fn expect_status(mut self, status: Status) -> Self {
        self.expected_status = status;
        self
    }

    pub fn test(mut self) {
        let mut config_data = HashMap::new();

        // Create temporary mock files
        for test_file in self.test_files.iter_mut() {
            // Create the file handle
            let mut file = tempfile::tempfile().unwrap();

            // Write the initial data to the file
            write!(file, "{}", test_file.initial_data).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();

            // Save the file for the test and verification
            config_data.insert(test_file.pihole_file, file.try_clone().unwrap());
            test_file.temp_file = Some(file);
        }

        // Start the test client
        let client = setup::test(self.ftl_data, config_data);

        // Make the request
        let mut request = client.req(self.method, self.endpoint);

        // Add the authentication header
        if self.should_auth {
            request.add_header(
                Header::new("X-Pi-hole-Authenticate", base64::encode("test_key"))
            );
        }

        // Add the rest of the headers
        for header in self.headers {
            request.add_header(header);
        }

        // Set the body data if necessary
        if let Some(data) = self.body_data {
            request.add_header(ContentType::JSON);
            request.set_body(serde_json::to_vec(&data).unwrap());
        }

        // Dispatch the request
        println!("{:#?}", request);
        let mut response = request.dispatch();
        println!("\nResponse:\n{:?}", response);

        // Check the status
        assert_eq!(self.expected_status, response.status());

        // Check that something was returned
        let body = response.body_string();
        assert!(body.is_some());

        // Check that it is correct JSON
        let parsed: serde_json::Value = serde_json::from_str(&body.unwrap()).unwrap();

        // Check that is is the same as the expected JSON
        assert_eq!(self.expected_json, parsed);

        // Verify files are as expected at the end
        let mut buffer = String::new();

        // Get the file handles and expected data
        let expected_data: Vec<(File, String)> = self.test_files.into_iter()
            .map(|test_file| {
                let expected = test_file.expected_data;
                let file = test_file.temp_file.unwrap();

                (file, expected)
            })
            .collect();

        // Check the files against the expected data
        for (mut file, expected) in expected_data {
            file.seek(SeekFrom::Start(0)).unwrap();
            file.read_to_string(&mut buffer).unwrap();

            assert_eq!(buffer, expected);
            buffer.clear();
        }
    }
}

/// Add the end of message byte to the data
pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}
