/* Pi-hole: A black hole for Internet advertisements
*  (c) 2018 Pi-hole, LLC (https://pi-hole.net)
*  Network-wide ad blocking via your own hardware.
*
*  API
*  Version endpoint
*
*  This file is copyright under the latest version of the EUPL.
*  Please see LICENSE file for your rights under this license. */

use rocket::State;
use config::Config;
use config::PiholeFile;
use ftl::FtlConnectionType;
use util;
use std::io::Read;

/// Get the versions of all Pi-hole systems
#[get("/version")]
pub fn version(config: State<Config>, ftl: State<FtlConnectionType>) -> util::Reply {
    // Core
    // Web
    // FTL
    // API
    let core_version = read_core_version(&config)?;

    util::reply_data(json!({
        "core": core_version,
    }))
}

/// Read Core version information from the file system
fn read_core_version(config: &Config) -> Result<Version, util::Error> {
    // Read the version files
    let mut local_versions = String::new();
    let mut local_branches = String::new();
    config.read_file(PiholeFile::LocalVersions)?
        .read_to_string(&mut local_versions)?;
    config.read_file(PiholeFile::LocalBranches)?
        .read_to_string(&mut local_branches)?;

    // These files are structured as "CORE WEB FTL", but we only want Core's data
    let git_version = local_versions.split_whitespace().next().unwrap_or_default();
    let core_branch = local_branches.split_whitespace().next().unwrap_or_default();

    // Parse the version data
    Ok(parse_version(git_version, core_branch)?)
}

/// Parse version data from the output of `git describe` (stored in `PiholeFile::LocalVersions`).
/// The string is in the form `TAG-NUMBER-COMMIT`.
fn parse_version(git_version: &str, branch: &str) -> Result<Version, util::Error> {
    let split: Vec<&str> = git_version.split("-").collect();

    if split.len() != 3 {
        return Err(util::Error::Unknown);
    }

    // Only set the tag if this is the tagged commit (we are 0 commits after the tag)
    let tag = if split[1] == "0" { split[0] } else { "" };

    Ok(Version {
        tag: tag.to_owned(),
        branch: branch.to_owned(),
        // Ignore the beginning "g" character
        hash: split[2].get(1..).unwrap_or_default().to_owned()
    })
}

#[derive(Debug, PartialEq, Serialize)]
struct Version {
    tag: String,
    branch: String,
    hash: String
}

#[cfg(test)]
mod tests {
    use super::{parse_version, Version};
    use util;
    use testing::TestConfigBuilder;
    use config::PiholeFile;
    use config::Config;
    use version::read_core_version;

    #[test]
    fn test_read_core_version_valid() {
        let test_config = Config::Test(
            TestConfigBuilder::new()
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
            read_core_version(&test_config),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "6689e00".to_owned()
            })
        )
    }

    #[test]
    fn test_read_core_version_invalid() {
        let test_config = Config::Test(
            TestConfigBuilder::new()
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
            read_core_version(&test_config),
            Err(util::Error::Unknown)
        )
    }

    #[test]
    fn test_parse_version_valid() {
        assert_eq!(
            parse_version("v3.3.1-0-gfbee18e", "master"),
            Ok(Version {
                tag: "v3.3.1".to_owned(),
                branch: "master".to_owned(),
                hash: "fbee18e".to_owned()
            })
        )
    }

    #[test]
    fn test_parse_version_old_tag() {
        assert_eq!(
            parse_version("v3.3.1-222-gd9c924b", "development"),
            Ok(Version {
                tag: "".to_owned(),
                branch: "development".to_owned(),
                hash: "d9c924b".to_owned()
            })
        )
    }

    #[test]
    fn test_parse_version_invalid() {
        assert_eq!(
            parse_version("invalid data", "branch"),
            Err(util::Error::Unknown)
        )
    }
}
