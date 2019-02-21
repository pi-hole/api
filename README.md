# Pi-hole API

Work in progress HTTP API for Pi-hole.
The API reads FTL's shared memory so it can directly read the statistics FTL
generates. This API is the replacement for most of FTL's socket/telnet API, as
well as the PHP API of the pre-5.0 web interface.

## Getting Started (Development)

- Install Rust: https://www.rust-lang.org/tools/install
    - Currently the project uses Rust nightly. The exact version used is stored
      in [`rust-toolchain`](rust-toolchain). The version should be detected and
      used automatically when you run a Rust command in the project directory,
      such as `cargo check` (this is a feature of `rustup`)
- Install your distro's build tools
    - `build-essential` for Debian distros, `gcc-c++` and `make` for RHEL
      distros
- Install libsqlite3
    - `libsqlite3-dev` for Debian distros, `sqlite-devel` for RHEL
- Fork the repository and clone to your computer (not the Pi-hole). In
  production the Pi-hole only needs the compiled output of the project, not its
  source code
    - Checkout the `development` branch for the latest changes.
- Run `cargo check`. This will download the Rust nightly toolchain and project
  dependencies, and it will check the program for errors. If everything was set
  up correctly, the final output should look like this:
  ```
      Finished dev [unoptimized + debuginfo] target(s) in 1m 11s
  ```
- Run `cargo test`. This will compile and run the tests. They should all pass
  :wink:
- If you've never used Rust, you should look at the [documentation][Rust Docs],
  including the [Rust Book], before diving too deep into the code.
- When you are ready to make changes, make a branch off of `development` in your
  fork to work in. When you're ready to make a pull request, base the PR against
  `development`.

[Rust Docs]: https://www.rust-lang.org/learn
[Rust Book]: https://doc.rust-lang.org/book/
