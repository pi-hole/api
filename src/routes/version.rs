// Pi-hole: A black hole for Internet advertisements
// (c) 2018 Pi-hole, LLC (https://pi-hole.net)
// Network-wide ad blocking via your own hardware.
//
// API
// Version Endpoint
//
// This file is copyright under the latest version of the EUPL.
// Please see LICENSE file for your rights under this license.

use env::{Env, PiholeFile};
use failure::ResultExt;
use ftl::FtlConnectionType;
use rocket::State;
use routes::web::WebAssets;
use std::io::Read;
use std::str;
use util::{reply_data, Error, ErrorKind, Reply};

/// Get the versions of all Pi-hole systems
#[get("/version")]
pub fn version(env: State<Env>, ftl: State<FtlConnectionType>) -> Reply {
    let core_version = read_core_version(&env).unwrap_or_default();
    let web_version = read_web_version().unwrap_or_default();
    let ftl_version = read_ftl_version(&ftl).unwrap_or_default();
    let api_version = read_api_version();

    reply_data(json!({
        "core": core_version,
        "web": web_version,
        "ftl": ftl_version,
        "api": api_version
    }))
}

/// Read API version information from the compile-time environment variables
fn read_api_version() -> Version {
    Version {
        tag: env!("GIT_TAG").to_owned(),
        branch: env!("GIT_BRANCH").to_owned(),
        hash: env!("GIT_HASH").get(0..7).unwrap_or_default().to_owned() // Use a short hash
    }
}

/// Read FTL version information from FTL's API
fn read_ftl_version(ftl: &FtlConnectionType) -> Result<Version, Error> {
    let mut con = ftl.connect("version")?;
    let mut str_buffer = [0u8; 4096];

    // Ignore the version and date strings
    let _hash_tag = con.read_str(&mut str_buffer)?.to_owned();
    let tag = con.read_str(&mut str_buffer)?.to_owned();
    let branch = con.read_str(&mut str_buffer)?.to_owned();
    let hash = con.read_str(&mut str_buffer)?.to_owned();
    let _date = con.read_str(&mut str_buffer)?.to_owned();
    con.expect_eom()?;

    Ok(Version { tag, branch, hash })
}

/// Read Web version information from the `VERSION` file in the web assets.
fn read_web_version() -> Result<Version, Error> {
    let version_raw = WebAssets::get("VERSION").ok_or(ErrorKind::Unknown)?;
    let version_str = str::from_utf8(&version_raw).context(ErrorKind::Unknown)?;

    parse_web_version(version_str)
}

/// Parse Web version information from the string.
/// The string should be in the format "TAG BRANCH COMMIT".
fn parse_web_version(version_str: &str) -> Result<Version, Error> {
    // Trim to remove possible newline
    let version_split: Vec<&str> = version_str.trim_right_matches("\n").split(" ").collect();

    if version_split.len() != 3 {
        return Err(Error::from(ErrorKind::Unknown));
    }

    Ok(Version {
        tag: version_split[0].to_owned(),
        branch: version_split[1].to_owned(),
        hash: version_split[2].to_owned()
    })
}

/// Read Core version information from the file system
fn read_core_version(env: &Env) -> Result<Version, Error> {
    // Read the version files
    let mut local_versions = String::new();
    let mut local_branches = String::new();
    env.read_file(PiholeFile::LocalVersions)?
        .read_to_string(&mut local_versions)
        .context(ErrorKind::FileRead(
            env.file_location(PiholeFile::LocalVersions).to_owned()
        ))?;
    env.read_file(PiholeFile::LocalBranches)?
        .read_to_string(&mut local_branches)
        .context(ErrorKind::FileRead(
            env.file_location(PiholeFile::LocalBranches).to_owned()
        ))?;

    // These files are structured as "CORE WEB FTL", but we only want Core's data
    let git_version = local_versions.split(" ").next().unwrap_or_default();
    let core_branch = local_branches.split(" ").next().unwrap_or_default();

    // Parse the version data
    parse_git_version(git_version, core_branch)
}

/// Parse version data from the output of `git describe` (stored in
/// `PiholeFile::LocalVersions`). The string is in the form
/// "TAG-NUMBER-COMMIT", though it could also have "-dirty" at the end.
fn parse_git_version(git_version: &str, branch: &str) -> Result<Version, Error> {
    let split: Vec<&str> = git_version.split("-").collect();

    // Could include "-dirty", which would make the length equal 4
    if split.len() < 3 {
        return Err(Error::from(ErrorKind::Unknown));
    }

    // Only set the tag if this is the tagged commit (we are 0 commits after the
    // tag)
    let tag = if split[1] == "0" { split[0] } else { "" };

    Ok(Version {
        tag: tag.to_owned(),
        branch: branch.to_owned(),
        // Ignore the beginning "g" character
        hash: split[2].get(1..).unwrap_or_default().to_owned()
    })
}

#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(Serialize, Default)]
struct Version {
    tag: String,
    branch: String,
    hash: String
}

#[cfg(test)]
mod tests {
    use super::{parse_git_version, parse_web_version, read_ftl_version, Version};
    use env::{Config, Env, PiholeFile};
    use ftl::FtlConnectionType;
    use rmp::encode;
    use routes::version::read_core_version;
    use std::collections::HashMap;
    use testing::{write_eom, TestEnvBuilder};
    use util::ErrorKind;

    #[test]
    fn test_read_ftl_version_dev() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "vDev-4d5da59").unwrap();
        encode::write_str(&mut data, "").unwrap();
        encode::write_str(&mut data, "tweak/version-api").unwrap();
        encode::write_str(&mut data, "4d5da59").unwrap();
        encode::write_str(&mut data, "2018-06-11 21:25:02 -0400").unwrap();
        write_eom(&mut data);

        let mut map = HashMap::new();
        map.insert("version".to_owned(), data);

        let ftl = FtlConnectionType::Test(map);

        assert_eq!(
            read_ftl_version(&ftl).map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "tweak/version-api".to_owned(),
                hash: "4d5da59".to_owned()
            })
        )
    }

    #[test]
    fn test_read_ftl_version_release() {
        let mut data = Vec::new();
        encode::write_str(&mut data, "v4.0").unwrap();
        encode::write_str(&mut data, "v4.0").unwrap();
        encode::write_str(&mut data, "master").unwrap();
        encode::write_str(&mut data, "abcdefg").unwrap();
        encode::write_str(&mut data, "2018-06-11 21:25:02 -0400").unwrap();
        write_eom(&mut data);

        let mut map = HashMap::new();
        map.insert("version".to_owned(), data);

        let ftl = FtlConnectionType::Test(map);

        assert_eq!(
            read_ftl_version(&ftl).map_err(|e| e.kind()),
            Ok(Version {
                tag: "v4.0".to_owned(),
                branch: "master".to_owned(),
                hash: "abcdefg".to_owned()
            })
        )
    }

    #[test]
    fn test_parse_web_version_dev() {
        assert_eq!(
            parse_web_version(" development d2037fd").map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "d2037fd".to_owned()
            })
        );
    }

    #[test]
    fn test_parse_web_version_release() {
        assert_eq!(
            parse_web_version("v1.0.0 master abcdefg").map_err(|e| e.kind()),
            Ok(Version {
                tag: "v1.0.0".to_owned(),
                branch: "master".to_owned(),
                hash: "abcdefg".to_owned()
            })
        );
    }

    #[test]
    fn test_parse_web_version_invalid() {
        assert_eq!(
            parse_web_version("invalid data").map_err(|e| e.kind()),
            Err(ErrorKind::Unknown)
        );
    }

    #[test]
    fn test_parse_web_version_newline() {
        assert_eq!(
            parse_web_version(" development d2037fd\n").map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "d2037fd".to_owned()
            })
        );
    }

    #[test]
    fn test_read_core_version_valid() {
        let test_env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(
                    PiholeFile::LocalVersions,
                    "v3.3.1-219-g6689e00 v3.3-190-gf7e1a28 vDev-d06deca"
                )
                .file(
                    PiholeFile::LocalBranches,
                    "development devel tweak/getClientNames"
                )
                .build()
        );

        assert_eq!(
            read_core_version(&test_env).map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "6689e00".to_owned()
            })
        );
    }

    #[test]
    fn test_read_core_version_invalid() {
        let test_env = Env::Test(
            Config::default(),
            TestEnvBuilder::new()
                .file(
                    PiholeFile::LocalVersions,
                    "invalid v3.3-190-gf7e1a28 vDev-d06deca"
                )
                .file(
                    PiholeFile::LocalBranches,
                    "development devel tweak/getClientNames"
                )
                .build()
        );

        assert_eq!(
            read_core_version(&test_env).map_err(|e| e.kind()),
            Err(ErrorKind::Unknown)
        );
    }

    #[test]
    fn test_parse_git_version_release() {
        assert_eq!(
            parse_git_version("v3.3.1-0-gfbee18e", "master").map_err(|e| e.kind()),
            Ok(Version {
                tag: "v3.3.1".to_owned(),
                branch: "master".to_owned(),
                hash: "fbee18e".to_owned()
            })
        );
    }

    #[test]
    fn test_parse_git_version_dev() {
        assert_eq!(
            parse_git_version("v3.3.1-222-gd9c924b", "development").map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "d9c924b".to_owned()
            })
        );
    }

    #[test]
    fn test_parse_git_version_invalid() {
        assert_eq!(
            parse_git_version("invalid data", "branch").map_err(|e| e.kind()),
            Err(ErrorKind::Unknown)
        );
    }

    #[test]
    fn test_parse_git_version_dirty() {
        assert_eq!(
            parse_git_version("v3.3.1-222-gd9c924b-dirty", "development").map_err(|e| e.kind()),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "d9c924b".to_owned()
            })
        );
    }
}
