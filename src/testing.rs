// Pi-hole: A black hole for Internet advertisements
// (c) 2019 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Common Test Functions
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use crate::{
    env::{Config, Env, PiholeFile},
    ftl::{FtlCounters, FtlMemory, FtlSettings},
    setup
};
use rocket::{
    http::{ContentType, Header, Method, Status},
    local::Client,
    Rocket
};
use std::{
    collections::HashMap,
    fs::File,
    io::{prelude::*, SeekFrom}
};
use tempfile::NamedTempFile;

/// Add the end of message byte to the data
pub fn write_eom(data: &mut Vec<u8>) {
    data.push(0xc1);
}

/// Builds the data needed to create a `Env::Test`
pub struct TestEnvBuilder {
    test_files: Vec<TestFile<NamedTempFile>>
}

impl TestEnvBuilder {
    /// Create a new `TestEnvBuilder`
    pub fn new() -> TestEnvBuilder {
        TestEnvBuilder {
            test_files: Vec::new()
        }
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
            NamedTempFile::new().unwrap(),
            initial_data.to_owned(),
            expected_data.to_owned()
        );
        self.test_files.push(test_file);
        self
    }

    /// Build the environment. This will create an `Env::Test` with a default
    /// config.
    pub fn build(self) -> Env {
        let mut env_data = HashMap::new();

        // Create temporary mock files
        for mut test_file in self.test_files {
            // Write the initial data to the file
            write!(test_file.temp_file, "{}", test_file.initial_data).unwrap();
            test_file.temp_file.seek(SeekFrom::Start(0)).unwrap();

            // Save the file for the test
            env_data.insert(test_file.pihole_file, test_file.temp_file);
        }

        Env::Test(Config::default(), env_data)
    }

    /// Get a copy of the inner test files for later verification
    pub fn clone_test_files(&self) -> Vec<TestFile<File>> {
        let mut test_files = Vec::new();

        for test_file in &self.test_files {
            test_files.push(TestFile {
                pihole_file: test_file.pihole_file,
                temp_file: test_file.temp_file.reopen().unwrap(),
                initial_data: test_file.initial_data.clone(),
                expected_data: test_file.expected_data.clone()
            })
        }

        test_files
    }
}

/// Represents a mocked file along with the initial and expected data. The `T`
/// generic is the type of temporary file, usually `NamedTempFile` or `File`.
pub struct TestFile<T: Seek + Read> {
    pihole_file: PiholeFile,
    temp_file: T,
    initial_data: String,
    expected_data: String
}

impl<T: Seek + Read> TestFile<T> {
    /// Create a new `TestFile`
    fn new(
        pihole_file: PiholeFile,
        temp_file: T,
        initial_data: String,
        expected_data: String
    ) -> TestFile<T> {
        TestFile {
            pihole_file,
            temp_file,
            initial_data,
            expected_data
        }
    }

    /// Asserts that the contents of the file matches the expected contents.
    /// `buffer` is used to read the file into memory, and will be cleared at
    /// the end.
    pub fn assert_expected(&mut self, buffer: &mut String) {
        self.temp_file.seek(SeekFrom::Start(0)).unwrap();
        self.temp_file.read_to_string(buffer).unwrap();

        assert_eq!(*buffer, self.expected_data);
        buffer.clear();
    }
}

/// Represents a test configuration, with all the data needed to carry out the
/// test
pub struct TestBuilder {
    endpoint: String,
    method: Method,
    headers: Vec<Header<'static>>,
    should_auth: bool,
    body_data: Option<serde_json::Value>,
    ftl_data: HashMap<String, Vec<u8>>,
    ftl_memory: FtlMemory,
    test_env_builder: TestEnvBuilder,
    expected_json: serde_json::Value,
    expected_status: Status,
    needs_database: bool,
    rocket_hooks: Vec<Box<dyn FnOnce(Rocket) -> Rocket>>
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
            ftl_memory: FtlMemory::Test {
                clients: Vec::new(),
                domains: Vec::new(),
                over_time: Vec::new(),
                queries: Vec::new(),
                upstreams: Vec::new(),
                strings: HashMap::new(),
                counters: FtlCounters::default(),
                settings: FtlSettings::default()
            },
            test_env_builder: TestEnvBuilder::new(),
            expected_json: json!({
                "data": [],
                "errors": []
            })
            .into(),
            expected_status: Status::Ok,
            needs_database: false,
            rocket_hooks: Vec::new()
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

    pub fn body<T: Into<serde_json::Value>>(mut self, body: T) -> Self {
        self.body_data = Some(body.into());
        self
    }

    pub fn ftl(mut self, command: &str, data: Vec<u8>) -> Self {
        self.ftl_data.insert(command.to_owned(), data);
        self
    }

    pub fn ftl_memory(mut self, ftl_memory: FtlMemory) -> Self {
        self.ftl_memory = ftl_memory;
        self
    }

    pub fn file(mut self, pihole_file: PiholeFile, initial_data: &str) -> Self {
        self.test_env_builder = self.test_env_builder.file(pihole_file, initial_data);
        self
    }

    pub fn file_expect(
        mut self,
        pihole_file: PiholeFile,
        initial_data: &str,
        expected_data: &str
    ) -> Self {
        self.test_env_builder =
            self.test_env_builder
                .file_expect(pihole_file, initial_data, expected_data);
        self
    }

    pub fn expect_json<T: Into<serde_json::Value>>(mut self, expected_json: T) -> Self {
        self.expected_json = expected_json.into();
        self
    }

    pub fn expect_status(mut self, status: Status) -> Self {
        self.expected_status = status;
        self
    }

    // This method is not used for now, but could be in the the future
    #[allow(unused)]
    pub fn need_database(mut self, need_database: bool) -> Self {
        self.needs_database = need_database;
        self
    }

    /// Add a function that will hook into Rocket's startup to add custom
    /// settings, such as state.
    pub fn add_rocket_hook(mut self, hook: impl FnOnce(Rocket) -> Rocket + 'static) -> Self {
        self.rocket_hooks.push(Box::new(hook));
        self
    }

    /// Add a struct into Rocket's state
    pub fn add_state(self, state: impl Send + Sync + 'static) -> Self {
        self.add_rocket_hook(move |rocket| rocket.manage(state))
    }

    /// Mock a service by adding it to Rocket's state. This is an alias of
    /// `add_state`
    pub fn mock_service(self, service: impl Send + Sync + 'static) -> Self {
        self.add_state(service)
    }

    pub fn test(self) {
        // Save the files for verification
        let test_files = self.test_env_builder.clone_test_files();

        // Configure the test server
        let mut rocket = setup::test(
            self.ftl_data,
            self.ftl_memory,
            self.test_env_builder.build(),
            self.needs_database
        );

        // Execute the Rocket hooks
        for hook in self.rocket_hooks {
            rocket = hook(rocket);
        }

        // Start the test client
        let client = Client::new(rocket).unwrap();

        // Create the request
        let mut request = client.req(self.method, self.endpoint);

        // Add the authentication header
        if self.should_auth {
            request.add_header(Header::new("X-Pi-hole-Authenticate", "test_key"));
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

        // Check the files against the expected data
        let mut buffer = String::new();
        for mut test_file in test_files {
            test_file.assert_expected(&mut buffer);
        }
    }
}
