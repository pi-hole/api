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

use config::PiholeFile;
use rocket::http::{Method, ContentType, Header, Status};
use setup;
use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::SeekFrom;

/// Add the end of message byte to the data
pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}

/// Builds the data needed to create a `Env::Test`
pub struct TestEnvBuilder {
    test_files: Vec<TestFile>
}

impl TestEnvBuilder {
    /// Create a new `TestEnvBuilder`
    pub fn new() -> TestEnvBuilder {
        TestEnvBuilder { test_files: Vec::new() }
    }

    /// Add a file and verify that it does not change
    pub fn file(self, pihole_file: PiholeFile, initial_data: &str) -> Self {
        self.file_expect(pihole_file, initial_data, initial_data)
    }

    /// Add a file and verify that it ends up in a certain state
    pub fn file_expect(
        mut self,
        pihole_file: PiholeFile,
        initial_data: &str,
        expected_data: &str
    ) -> Self {
        let test_file = TestFile::new(
            pihole_file,
            tempfile::tempfile().unwrap(),
            initial_data.to_owned(),
            expected_data.to_owned()
        );
        self.test_files.push(test_file);
        self
    }

    /// Build the environment data. This can be used to create a `Env::Test`
    pub fn build(self) -> HashMap<PiholeFile, File> {
        let mut config_data = HashMap::new();

        // Create temporary mock files
        for mut test_file in self.test_files {
            // Write the initial data to the file
            write!(test_file.temp_file, "{}", test_file.initial_data).unwrap();
            test_file.temp_file.seek(SeekFrom::Start(0)).unwrap();

            // Save the file for the test
            config_data.insert(test_file.pihole_file, test_file.temp_file);
        }

        config_data
    }

    /// Get a copy of the inner test files for later verification
    fn get_test_files(&self) -> Vec<TestFile> {
        let mut test_files = Vec::new();

        for test_file in &self.test_files {
            test_files.push(TestFile {
                pihole_file: test_file.pihole_file,
                temp_file: test_file.temp_file.try_clone().unwrap(),
                initial_data: test_file.initial_data.clone(),
                expected_data: test_file.expected_data.clone()
            })
        }

        test_files
    }
}

/// Represents a mocked file along with the initial and expected data
struct TestFile {
    pihole_file: PiholeFile,
    temp_file: File,
    initial_data: String,
    expected_data: String
}

impl TestFile {
    /// Create a new `TestFile`
    fn new(
        pihole_file: PiholeFile,
        temp_file: File,
        initial_data: String,
        expected_data: String
    ) -> TestFile {
        TestFile {
            pihole_file,
            temp_file,
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
    test_config_builder: TestEnvBuilder,
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
            test_config_builder: TestEnvBuilder::new(),
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

    pub fn file(mut self, pihole_file: PiholeFile, initial_data: &str) -> Self {
        self.test_config_builder = self.test_config_builder.file(pihole_file, initial_data);
        self
    }

    pub fn file_expect(
        mut self,
        pihole_file: PiholeFile,
        initial_data: &str,
        expected_data: &str
    ) -> Self {
        self.test_config_builder = self.test_config_builder
            .file_expect(pihole_file, initial_data, expected_data);
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

    pub fn test(self) {
        // Save the files for verification
        let test_files = self.test_config_builder.get_test_files();

        // Start the test client
        let config_data = self.test_config_builder.build();
        let client = setup::test(self.ftl_data, config_data);

        // Create the request
        let mut request = client.req(self.method, self.endpoint);

        // Add the authentication header
        if self.should_auth {
            request.add_header(
                Header::new("X-Pi-hole-Authenticate", "test_key")
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

        let body_str = body.unwrap();
        println!("Body:\n{}", body_str);

        // Check that it is correct JSON
        let parsed: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        // Check that is is the same as the expected JSON
        assert_eq!(self.expected_json, parsed);

        // Verify files are as expected at the end
        let mut buffer = String::new();

        // Get the file handles and expected data
        let expected_data: Vec<(File, String)> = test_files.into_iter()
            .map(|test_file| (test_file.temp_file, test_file.expected_data))
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
