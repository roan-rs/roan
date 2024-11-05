use semver::{Comparator, Op, Version, VersionReq};

pub fn version_to_req(version: Version, default: Op) -> VersionReq {
    VersionReq {
        comparators: vec![Comparator {
            major: version.major,
            minor: Some(version.minor),
            patch: Some(version.patch),
            pre: version.pre.clone(),
            op: default,
        }],
    }
}
