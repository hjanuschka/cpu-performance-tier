// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! CPU performance tier classification.
//!
//! This crate is a Rust port of Chromium's experimental CPU Performance API
//! classifier. By default it contains only the pure classification logic.
//! Enable the `host-info` feature for dependency-free helpers that read host
//! CPU information through platform facilities.

use regex::Regex;

mod generated;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PerformanceTier {
    Unknown = 0,
    Low = 1,
    Mid = 2,
    High = 3,
    Ultra = 4,
}

impl TryFrom<i32> for PerformanceTier {
    type Error = TierFromIntError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::Low),
            2 => Ok(Self::Mid),
            3 => Ok(Self::High),
            4 => Ok(Self::Ultra),
            _ => Err(TierFromIntError(value)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TierFromIntError(pub i32);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Manufacturer {
    Unknown,
    Amd,
    Apple,
    Intel,
    MediaTek,
    Microsoft,
    Qualcomm,
    Samsung,
}

#[cfg(feature = "host-info")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostCpuInfo {
    pub cpu_model: Option<String>,
    pub logical_cores: Option<usize>,
}

fn re(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|err| panic!("invalid regex {pattern:?}: {err}"))
}

fn search(text: &str, pattern: &str) -> bool {
    re(pattern).is_match(text)
}

fn replace(text: &mut String, pattern: &str, replacement: &str) {
    *text = re(pattern).replace_all(text, replacement).into_owned();
}

fn replace_first(text: &mut String, pattern: &str, replacement: &str) {
    *text = re(pattern).replace(text, replacement).into_owned();
}

fn trim_and_collapse_whitespace(text: &mut String) {
    // \p{Z} matches unicode separators, including NBSP. \x1C-\x1F match the
    // file, group, record, and unit separator characters.
    replace(text, r"^[\s\p{Z}\x{1C}-\x{1F}]+", "");
    replace(text, r"[\s\p{Z}\x{1C}-\x{1F}]+$", "");
    replace(text, r"[\s\p{Z}\x{1C}-\x{1F}]+", " ");
}

fn manufacturer_from_model(cpu_model: &str) -> Manufacturer {
    if search(cpu_model, r"(?i)\bAMD\b") {
        Manufacturer::Amd
    } else if search(cpu_model, r"(?i)\bApple\b") {
        Manufacturer::Apple
    } else if search(cpu_model, r"(?i)\b(Intel|Celeron|Pentium)\b") {
        Manufacturer::Intel
    } else if search(cpu_model, r"(?i)\bMediaTek\b") {
        Manufacturer::MediaTek
    } else if search(cpu_model, r"(?i)\bMicrosoft\b") {
        Manufacturer::Microsoft
    } else if search(cpu_model, r"(?i)\b(Qualcomm|Snapdragon)\b") {
        Manufacturer::Qualcomm
    } else if search(cpu_model, r"(?i)\bSamsung\b") {
        Manufacturer::Samsung
    } else {
        Manufacturer::Unknown
    }
}

/// Returns the detected manufacturer and normalized model string.
pub fn split_cpu_model(cpu_model: &str) -> (Manufacturer, String) {
    let mut text = cpu_model.to_string();

    trim_and_collapse_whitespace(&mut text);

    let manufacturer = manufacturer_from_model(&text);

    replace(&mut text, r"\([^)]*\)", " ");
    replace(&mut text, r"\$|®|™", " ");
    replace(
        &mut text,
        r"(?i)@( )?\d[.,]\d+([~-]\d[.,]\d+)?( )?GHz\b",
        "",
    );
    replace(&mut text, r"(?i)\b\d[.,]\d+([~-]\d[.,]\d+)?( )?GHz\b", "");

    trim_and_collapse_whitespace(&mut text);

    replace(&mut text, r"(^| )?[@~\-,.]$", "");

    replace(&mut text, r"(?i)\bCPU\b", "");
    replace(&mut text, r"(?i)\bMobile\b", "");
    replace(&mut text, r"(?i)\bProcessor\b", "");
    replace(&mut text, r"(?i)\bSilicon\b", "");
    replace(&mut text, r"(?i)\bSOC\b", "");
    replace(&mut text, r"(?i)\bTechnology\b", "");

    trim_and_collapse_whitespace(&mut text);

    match manufacturer {
        Manufacturer::Amd => {
            replace_first(&mut text, r"(?i).*?\bAMD\b", "");
            trim_and_collapse_whitespace(&mut text);
            replace(&mut text, r"(?i)\bFX -", "FX-");
            replace(&mut text, r"(?i)\+( )?(AMD )?Radeon.*", "");
            replace(
                &mut text,
                r"(?i)\b(RADEON )?R\d+, \d+ COMPUTE CORES \d+C\+\d+G\b",
                "",
            );
            replace(&mut text, r"(?i)\bwith (AMD )?Radeon.*", "");
            replace(&mut text, r"(?i)\bw/( )?(AMD )?Radeon.*", "");
            replace(&mut text, r"(?i)\bRadeon.*", "");
            replace(&mut text, r"(?i)\b\w+( |-)Core\b", "");
            replace(&mut text, r"(?i)\b\d+-Core(s)?\b", "");
            replace(&mut text, r"(?i)\bAPU\b", "");
            replace(&mut text, r"(?i)\bCreator Edition\b", "");
            replace(&mut text, r"(?i)\bDesktop Kit\b", "");
            replace(&mut text, r"(?i)\b(3250C) 15W\b", "$1");
        }
        Manufacturer::Apple => {
            replace_first(&mut text, r"(?i).*?\bApple\b", "");
        }
        Manufacturer::Intel => {
            replace_first(&mut text, r"(?i).*?\bIntel\b", "");
            trim_and_collapse_whitespace(&mut text);
            replace(&mut text, r"(?i)\b(Core)(2)\b", "${1} ${2}");
            replace(&mut text, r"(?i)\b(Core i\d+)( )?-( )?", "${1}-");
            replace(&mut text, r"(?i)\b(Core i\d+) (M) (\d+)\b", "${1}-${3}${2}");
            replace(
                &mut text,
                r"(?i)\b(Core i\d+) ([LQU]) (\d+)\b",
                "${1}-${3}${2}M",
            );
            replace(&mut text, r"(?i)\b(Core i\d+) (\d+)\b", "${1}-${2}");
            replace(&mut text, r"(?i)\b(Celeron|Pentium) Dual(-Core)?\b", "${1}");
            replace(&mut text, r"\b0+$", "");
        }
        _ => return (manufacturer, String::new()),
    }

    trim_and_collapse_whitespace(&mut text);
    (manufacturer, text)
}

/// Returns the fallback tier based only on the number of logical processors.
pub fn tier_from_cores(cores: i32) -> PerformanceTier {
    match cores {
        1..=2 => PerformanceTier::Low,
        3..=4 => PerformanceTier::Mid,
        5..=12 => PerformanceTier::High,
        13.. => PerformanceTier::Ultra,
        _ => PerformanceTier::Unknown,
    }
}

/// Returns the performance tier from a CPU model string and logical core count.
pub fn tier_from_cpu_info(cpu_model: &str, cores: i32) -> PerformanceTier {
    if cores <= 0 {
        return PerformanceTier::Unknown;
    }
    if cores <= 1 {
        return PerformanceTier::Low;
    }

    let (manufacturer, model) = split_cpu_model(cpu_model);

    for rule in generated::PATTERN_RULES {
        if rule.manufacturer == manufacturer
            && cores >= rule.min_cores
            && cores <= rule.max_cores
            && rule.include.iter().any(|pattern| search(&model, pattern))
            && !rule.exclude.iter().any(|pattern| search(&model, pattern))
        {
            return rule.tier;
        }
    }

    if cores <= 4 {
        if !matches!(manufacturer, Manufacturer::Amd | Manufacturer::Intel) && cores <= 2 {
            return PerformanceTier::Low;
        }
        return PerformanceTier::Mid;
    }

    if cores <= 10 {
        return PerformanceTier::High;
    }

    PerformanceTier::Ultra
}

#[cfg(feature = "host-info")]
fn usize_to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Returns the host logical processor count.
///
/// This helper is available with the `host-info` feature. It uses only the
/// Rust standard library.
#[cfg(feature = "host-info")]
pub fn host_logical_cores() -> Option<usize> {
    std::thread::available_parallelism().ok().map(usize::from)
}

/// Returns the host CPU model or brand string when available.
///
/// This helper is available with the `host-info` feature and does not add any
/// crate dependencies. It uses `/proc/cpuinfo` on Linux and Android, `sysctl`
/// on macOS-like platforms, and environment variables on Windows.
#[cfg(feature = "host-info")]
pub fn host_cpu_model() -> Option<String> {
    host_cpu_model_impl()
}

/// Returns both host inputs needed by the classifier.
#[cfg(feature = "host-info")]
pub fn host_cpu_info() -> HostCpuInfo {
    HostCpuInfo {
        cpu_model: host_cpu_model(),
        logical_cores: host_logical_cores(),
    }
}

/// Returns the host performance tier using the CPU model when available and
/// falling back to logical core count otherwise.
#[cfg(feature = "host-info")]
pub fn tier_from_host() -> PerformanceTier {
    let info = host_cpu_info();
    match (info.cpu_model.as_deref(), info.logical_cores) {
        (Some(cpu_model), Some(logical_cores)) => {
            tier_from_cpu_info(cpu_model, usize_to_i32(logical_cores))
        }
        (_, Some(logical_cores)) => tier_from_cores(usize_to_i32(logical_cores)),
        _ => PerformanceTier::Unknown,
    }
}

#[cfg(all(feature = "host-info", any(target_os = "linux", target_os = "android")))]
fn host_cpu_model_impl() -> Option<String> {
    cpu_model_from_proc_cpuinfo(&std::fs::read_to_string("/proc/cpuinfo").ok()?)
}

#[cfg(all(
    feature = "host-info",
    any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd"
    )
))]
fn host_cpu_model_impl() -> Option<String> {
    let output = std::process::Command::new("sysctl")
        .args(["-n", "machdep.cpu.brand_string"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    non_empty_line(String::from_utf8_lossy(&output.stdout).trim())
}

#[cfg(all(feature = "host-info", target_os = "windows"))]
fn host_cpu_model_impl() -> Option<String> {
    non_empty_line(&std::env::var("PROCESSOR_IDENTIFIER").ok()?)
}

#[cfg(all(
    feature = "host-info",
    not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "windows"
    ))
))]
fn host_cpu_model_impl() -> Option<String> {
    None
}

#[cfg(feature = "host-info")]
fn non_empty_line(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

#[cfg(feature = "host-info")]
fn cpu_model_from_proc_cpuinfo(contents: &str) -> Option<String> {
    for key in ["model name", "Processor", "Hardware"] {
        for line in contents.lines() {
            let Some((name, value)) = line.split_once(':') else {
                continue;
            };
            if name.trim().eq_ignore_ascii_case(key) {
                if let Some(value) = non_empty_line(value) {
                    return Some(value);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tier_from_cores() {
        for &(cores, expected_tier) in generated::CORE_TESTS {
            assert_eq!(
                expected_tier,
                tier_from_cores(cores),
                "failed for {cores} core(s)"
            );
        }
    }

    #[test]
    fn split_cpu_model_matches_chromium() {
        for &(cpu_model, expected_manufacturer, expected_model) in generated::SPLIT_TESTS {
            let (manufacturer, model) = split_cpu_model(cpu_model);
            assert_eq!(
                expected_manufacturer, manufacturer,
                "failed for '{cpu_model}'"
            );
            assert_eq!(expected_model, model, "failed for '{cpu_model}'");
        }
    }

    #[test]
    fn get_tier_from_cpu_info_matches_chromium() {
        for &(cores, model, expected_tier) in generated::TIER_CPU_INFO_TESTS {
            assert_eq!(
                expected_tier,
                tier_from_cpu_info(model, cores),
                "failed for '{model}' with {cores} core(s)"
            );
        }
    }

    #[test]
    fn tier_from_int() {
        assert_eq!(PerformanceTier::try_from(0), Ok(PerformanceTier::Unknown));
        assert_eq!(PerformanceTier::try_from(1), Ok(PerformanceTier::Low));
        assert_eq!(PerformanceTier::try_from(2), Ok(PerformanceTier::Mid));
        assert_eq!(PerformanceTier::try_from(3), Ok(PerformanceTier::High));
        assert_eq!(PerformanceTier::try_from(4), Ok(PerformanceTier::Ultra));
        assert_eq!(PerformanceTier::try_from(-1), Err(TierFromIntError(-1)));
        assert_eq!(PerformanceTier::try_from(5), Err(TierFromIntError(5)));
    }

    #[cfg(feature = "host-info")]
    #[test]
    fn parses_proc_cpuinfo_model() {
        let contents = "processor: 0\nmodel name: Intel(R) Core(TM) Ultra 7 155H\n";
        assert_eq!(
            cpu_model_from_proc_cpuinfo(contents),
            Some("Intel(R) Core(TM) Ultra 7 155H".to_string())
        );
    }

    #[cfg(feature = "host-info")]
    #[test]
    fn host_helpers_are_callable() {
        let _ = host_logical_cores();
        let _ = host_cpu_model();
        let _ = host_cpu_info();
        let _ = tier_from_host();
    }
}
