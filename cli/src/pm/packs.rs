use anyhow::{bail, format_err, Result};
use semver::{Comparator, Op, VersionReq};
use crate::pm::semver::version_to_req;

pub type PackVersion = (String, Option<VersionReq>);

pub fn parse_pack(pack: &str) -> Result<PackVersion> {
    let (pack, version) = if let Some((k, v)) = pack.split_once('@') {
        if k.is_empty() {
            bail!("Expected to have a crate name before '@'");
        }
        (k.to_owned(), Some(parse_semver_flag(v)?))
    } else {
        (pack.to_owned(), None)
    };

    if pack.is_empty() {
        bail!("crate name is empty");
    }

    Ok((pack, version))
}

fn parse_semver_flag(v: &str) -> Result<VersionReq> {
    let first = v
        .chars()
        .next()
        .ok_or_else(|| format_err!("version is missing"))?;

    if "<>=^~".contains(first) || v.contains('*') {
        v.parse::<VersionReq>()
            .map_err(|_| format_err!("Provided version '{}' is not a valid semver", v))
    } else {
        v.trim()
            .parse::<semver::Version>()
            .map(|v| version_to_req(v, Op::Exact))
            .map_err(|err| format_err!("Provided version '{}' is not a valid semver: {}", v, err))
    }
}
