// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use std::sync::LazyLock;

use libbarto::{Garuda, Pacman};
use regex::Regex;

// CachyOS
// cachyos-extra-v3/bind      9.20.13-1.1  9.20.15-1.1    0.01 MiB       2.21 MiB
// cachyos-core-v3/gc         8.2.10-1.1   8.2.10-2.1     0.00 MiB       0.24 MiB
// cachyos-extra-v3/libdecor  0.2.3-1.1    0.2.4-1.1      0.00 MiB       0.05 MiB
// cachyos-extra-v3/pcsclite  2.4.0-2.1    2.4.0-3.1      0.00 MiB       0.10 MiB

// pacman
// Packages (5) git-2.51.1-1  libarchive-3.8.2-1  linux-6.17.3.arch1-1  python-charset-normalizer-3.4.4-1  python-cryptography-46.0.3-1
// Packages (2) dhcpcd-10.2.4-1  libxml2-2.15.1-1
//
// Total Download Size:   0.96 MiB
// Total Installed Size:  3.45 MiB
// Net Upgrade Size:      0.00 MiB

// apt
// The following packages will be upgraded:
//   libtdb-dev libtdb1"
// 2 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.

// homebrew
// ==> Upgrading 2 outdated packages:
// protobuf 32.1 -> 33.0
// ==> Fetching downloads for: openexr and protobuf
// ==> Fetching openexr
// ==> Downloading https://ghcr.io/v2/homebrew/core/openexr/blobs/sha256:6e4279cef58092ba7d95c6f805b77ca4a8e3420010b0093d17d5ce058b749fd7
// ==> Fetching protobuf
// ==> Downloading https://ghcr.io/v2/homebrew/core/protobuf/blobs/sha256:220d0c9358fda8b85ce23cbb53596547f63895480c16498a6d3b8031710d4b21
// ==> Upgrading openexr
// "  3.4.1 -> 3.4.2 "
// ==> Pouring openexr--3.4.2.arm64_sequoia.bottle.tar.gz
// 🍺  /opt/homebrew/Cellar/openexr/3.4.2: 212 files, 4.9MB
// Removing: /opt/homebrew/Cellar/openexr/3.4.1... (212 files, 4.9MB)
// "  32.1 -> 33.0 "
// ==> Pouring protobuf--33.0.arm64_sequoia.bottle.tar.gz
// 🍺  /opt/homebrew/Cellar/protobuf/33.0: 364 files, 16.5MB

static GARUDA_UPDATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(chaotic-aur|core|extra|multilib)\/([^ ]+)\s+([^ ]+)\s+([^ ]+)\s+(.+ MiB)\s+(.+ MiB)",
    )
    .expect("failed to create garuda-update regex")
});
static PACMAN_PACKAGES_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Packages \((\d+)\) (.*)").expect("failed to create pacman packages update regex")
});
static PACMAN_DOWNLOAD_SIZE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Total Download Size:[ ]+(\d+\.\d+) MiB")
        .expect("failed to create pacman download size regex")
});
static PACMAN_INSTALL_SIZE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Total Installed Size:[ ]+(\d+\.\d+) MiB")
        .expect("failed to create pacman install size regex")
});
static NET_UPGRADE_SIZE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"Net Upgrade Size:[ ]+(\d+\.\d+) MiB")
        .expect("failed to create net upgrade size regex")
});
static CACHYOS_UPDATE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(cachyos-.*|core|extra|multilib)\/([^ ]+)\s+([^ ]+)\s+([^ ]+)\s+(.+ MiB)\s+(.+ MiB)",
    )
    .expect("failed to create cachyos-update regex")
});

pub(crate) fn garuda_filter(data: Vec<String>) -> Vec<Garuda> {
    let mut results = data
        .into_iter()
        .filter_map(|s| {
            GARUDA_UPDATE_RE.captures(&s).map(|caps| {
                Garuda::builder()
                    .channel(caps.get(1).map_or("", |m| m.as_str()))
                    .package(caps.get(2).map_or("", |m| m.as_str()))
                    .old_version(caps.get(3).map_or("", |m| m.as_str()))
                    .new_version(caps.get(4).map_or("", |m| m.as_str()))
                    .size_change(caps.get(5).map_or("", |m| m.as_str()))
                    .download_size(caps.get(6).map_or("", |m| m.as_str()))
                    .build()
            })
        })
        .collect::<Vec<Garuda>>();
    results.sort();
    results
}

pub(crate) fn pacman_filter(data: &[String]) -> Pacman {
    let (package_count, packages) = data
        .iter()
        .filter_map(|s| {
            PACMAN_PACKAGES_RE.captures(s).map(|caps| {
                (
                    caps.get(1)
                        .map_or(0, |m| m.as_str().parse::<usize>().unwrap_or(0)),
                    caps.get(2)
                        .map_or("", |m| m.as_str())
                        .split_whitespace()
                        .map(ToString::to_string)
                        .collect::<Vec<String>>(),
                )
            })
        })
        .fold((0, vec![]), |mut acc, (count, packages)| {
            acc.0 += count;
            acc.1.extend(packages);
            acc
        });

    let total_download_size = data
        .iter()
        .filter_map(|s| {
            PACMAN_DOWNLOAD_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    let total_install_size = data
        .iter()
        .filter_map(|s| {
            PACMAN_INSTALL_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    let net_install_size = data
        .iter()
        .filter_map(|s| {
            NET_UPGRADE_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    Pacman::builder()
        .update_count(package_count)
        .packages(packages)
        .install_size(total_install_size)
        .net_size(net_install_size)
        .download_size(total_download_size)
        .build()
}

pub(crate) fn cachyos_filter(data: &[String]) -> Pacman {
    let packages = data
        .iter()
        .filter_map(|s| {
            CACHYOS_UPDATE_RE.captures(s).map(|caps| {
                caps.get(2)
                    .map_or("", |m| m.as_str())
                    .split_whitespace()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
            })
        })
        .fold(vec![], |mut acc, packages| {
            acc.extend(packages);
            acc
        });

    let total_download_size = data
        .iter()
        .filter_map(|s| {
            PACMAN_DOWNLOAD_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    let total_install_size = data
        .iter()
        .filter_map(|s| {
            PACMAN_INSTALL_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    let net_install_size = data
        .iter()
        .filter_map(|s| {
            NET_UPGRADE_SIZE_RE.captures(s).map(|caps| {
                caps.get(1)
                    .map_or(0.0, |m| m.as_str().parse::<f64>().unwrap_or(0.0))
            })
        })
        .sum::<f64>();

    Pacman::builder()
        .update_count(packages.len())
        .packages(packages)
        .install_size(total_install_size)
        .net_size(net_install_size)
        .download_size(total_download_size)
        .build()
}

#[cfg(test)]
mod test {
    use super::GARUDA_UPDATE_RE;

    use anyhow::Result;

    const NO_MATCH: &str = "this is not a match";

    #[test]
    fn test_garuda_update_re_no_match() {
        assert!(!GARUDA_UPDATE_RE.is_match(NO_MATCH));
    }

    #[test]
    fn test_package_update_re() -> Result<()> {
        let text = "extra/kio    6.19.0-1     6.19.0-2       0.00 MiB       3.59 MiB";
        assert!(GARUDA_UPDATE_RE.is_match(text));
        let caps = GARUDA_UPDATE_RE
            .captures(text)
            .ok_or(anyhow::anyhow!("failed to capture"))?;
        assert_eq!(caps.get(1).map(|m| m.as_str()), Some("extra"));
        assert_eq!(caps.get(2).map(|m| m.as_str()), Some("kio"));
        assert_eq!(caps.get(3).map(|m| m.as_str()), Some("6.19.0-1"));
        assert_eq!(caps.get(4).map(|m| m.as_str()), Some("6.19.0-2"));
        assert_eq!(caps.get(5).map(|m| m.as_str()), Some("0.00 MiB"));
        assert_eq!(caps.get(6).map(|m| m.as_str()), Some("3.59 MiB"));
        Ok(())
    }
}
