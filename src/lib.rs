// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

//! CPU performance tier classification.
//!
//! This crate is a Rust port of Chromium's experimental CPU Performance API
//! classifier. It intentionally contains only the pure classification logic.

use regex::Regex;

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

    if cores <= 4 {
        match manufacturer {
            Manufacturer::Amd => {
                if cores == 2
                    && (search(&model, r"^Athlon 64\b")
                        || search(&model, r"^Athlon II\b")
                        || search(&model, r"^Athlon X2\b")
                        || search(&model, r"^Phenom II\b")
                        || search(&model, r"^Sempron X2\b")
                        || search(&model, r"^Turion II\b")
                        || search(&model, r"^Turion X2\b")
                        || search(&model, r"^(A4|E2)-[3]\d\d\d[A-Z]*\b")
                        || search(&model, r"^(A4|A6)-[4]\d\d\dM[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2
                    && (search(&model, r"^(C|E|E1|E2|T|Z)-\w*\b")
                        || search(&model, r"^(A4)-[1]\d\d\d[A-Z]*\b")
                        || search(&model, r"^(GX)-[2]\d\d[A-Z]*\b")
                        || search(&model, r"^Sempron 2650\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2 && search(&model, r"^A4-9120[Ce]\b") {
                    return PerformanceTier::Low;
                }
                if cores == 4
                    && search(&model, r"^Ryzen\b")
                    && !search(&model, r"^Ryzen 3 Pro 2100GE\b")
                    && !search(&model, r"^Ryzen 3 Pro 3050GE\b")
                    && !search(&model, r"^Ryzen 3 2200U\b")
                    && !search(&model, r"^Ryzen 3 3200U\b")
                    && !search(&model, r"^Ryzen 3 3250[UC]\b")
                    && !search(&model, r"^Ryzen Embedded R[1]\d\d\d[A-Z]*\b")
                    && !search(&model, r"^Ryzen Embedded R2312\b")
                {
                    return PerformanceTier::High;
                }
            }
            Manufacturer::Intel => {
                if cores >= 2
                    && cores <= 4
                    && (search(&model, r"^Atom (Z5)\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (2|3|N2)\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (D4|N4|D5|N5)\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (E6)\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (Z6)\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (D2|N2)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (Z2)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores >= 2
                    && cores <= 4
                    && (search(&model, r"^Celeron (J1)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (J2)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (C2)[35]\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (E3)[8]\d\d[A-Z]*\b")
                        || search(&model, r"^Celeron (N2)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (N3)[5]\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (Z3)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Atom x[5]-(E8)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Celeron (J3|N3)[01]\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (J3|N3)[7]\d\d[A-Z]*\b")
                        || search(&model, r"^Atom x[57]-(Z8)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2
                    && (search(&model, r"^Atom x[5]-(A3|E3)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Celeron (J3|N3)[34]\d\d[A-Z]*\b")
                        || search(&model, r"^Atom (C3)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2
                    && (search(&model, r"^Celeron (E1|SU2|T1|T3)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Core 2 Duo\b")
                        || search(&model, r"^Core 2 Extreme\b")
                        || search(&model, r"^Pentium (E2|SU2|SU4|T2|T3|T4)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Xeon (3|E3|L3)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2
                    && (search(&model, r"^Celeron (P4|U3)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (P6|U5)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 2
                    && (search(&model, r"^Celeron (7|8|B7|B8)\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (9|B9)\d\d[A-Z]*\b")
                        || search(&model, r"^Celeron (1)\d\d\d[A-Z]*\b")
                        || search(&model, r"^Pentium (2|A1)\d\d\d[A-Z]*\b"))
                {
                    return PerformanceTier::Low;
                }
                if cores == 4 && (search(&model, r"^N\d\d+\b") || search(&model, r"^Atom x7425E\b"))
                {
                    return PerformanceTier::High;
                }
            }
            _ => {
                if cores <= 2 {
                    return PerformanceTier::Low;
                }
            }
        }
        return PerformanceTier::Mid;
    }

    if cores <= 10 {
        match manufacturer {
            Manufacturer::Apple => {
                if cores >= 8 && search(&model, r"^M\d+\b") {
                    return PerformanceTier::Ultra;
                }
            }
            Manufacturer::Intel => {
                if cores >= 8 && search(&model, r"^Core Ultra\b") {
                    return PerformanceTier::Ultra;
                }
            }
            _ => {}
        }
        return PerformanceTier::High;
    }

    PerformanceTier::Ultra
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tier_from_cores() {
        let tests = [
            (1, PerformanceTier::Low),
            (2, PerformanceTier::Low),
            (3, PerformanceTier::Mid),
            (4, PerformanceTier::Mid),
            (5, PerformanceTier::High),
            (8, PerformanceTier::High),
            (12, PerformanceTier::High),
            (13, PerformanceTier::Ultra),
            (16, PerformanceTier::Ultra),
            (96, PerformanceTier::Ultra),
            (0, PerformanceTier::Unknown),
            (-42, PerformanceTier::Unknown),
        ];

        for (cores, expected_tier) in tests {
            assert_eq!(
                expected_tier,
                tier_from_cores(cores),
                "failed for {cores} core(s)"
            );
        }
    }

    #[test]
    fn split_cpu_model_matches_chromium() {
        let tests = [
            (
                "Intel(R) Core(TM) i7-10700K CPU @ 3.80GHz",
                Manufacturer::Intel,
                "Core i7-10700K",
            ),
            (
                "Intel® Core™i7-8600HQ CPU @ 3.0GHz",
                Manufacturer::Intel,
                "Core i7-8600HQ",
            ),
            (
                "Intel(R) Processor 5Y70 CPU @ 1.10GHz",
                Manufacturer::Intel,
                "5Y70",
            ),
            (
                "Intel(R) Core(TM) i3 CPU       M 330  @ 2.13GHz",
                Manufacturer::Intel,
                "Core i3-330M",
            ),
            (
                "Intel(R) Core(TM)2 Duo CPU     E4300  @ 1.80GHz",
                Manufacturer::Intel,
                "Core 2 Duo E4300",
            ),
            (
                "Intel(R) Core(TM) i7 CPU       L 620  @ 2.00GHz",
                Manufacturer::Intel,
                "Core i7-620LM",
            ),
            (
                "Intel(R) Core i5 - 10500u (tm) Processor",
                Manufacturer::Intel,
                "Core i5-10500u",
            ),
            (
                "Celeron(R) Dual-Core CPU       T3000  @ 1.80GHz",
                Manufacturer::Intel,
                "Celeron T3000",
            ),
            (
                "AMD Ryzen 7 5800X 8-Core Processor",
                Manufacturer::Amd,
                "Ryzen 7 5800X",
            ),
            (
                "AMD Ryzen 5 3500U with Radeon Vega Mobile Gfx",
                Manufacturer::Amd,
                "Ryzen 5 3500U",
            ),
            (
                "AMD A4-9120e RADEON R3, 4 COMPUTE CORES 2C+2G",
                Manufacturer::Amd,
                "A4-9120e",
            ),
            (
                "AMD A10-4600M APU with Radeon(tm) HD Graphics",
                Manufacturer::Amd,
                "A10-4600M",
            ),
            (
                "AMD FX(tm)-4130 Quad-Core Processor",
                Manufacturer::Amd,
                "FX-4130",
            ),
            (
                "AMD Ryzen 3 3250C 15W with Radeon Graphics",
                Manufacturer::Amd,
                "Ryzen 3 3250C",
            ),
            (
                "AMD Ryzen 5 6600HS Creator Edition",
                Manufacturer::Amd,
                "Ryzen 5 6600HS",
            ),
            (
                "AMD Turion(tm) 64 Mobile Technology MK-36",
                Manufacturer::Amd,
                "Turion 64 MK-36",
            ),
            (
                "AMD® Ryzen™5  3450U Quad Core@",
                Manufacturer::Amd,
                "Ryzen 5 3450U",
            ),
            ("Apple M1 ", Manufacturer::Apple, "M1"),
            ("Apple M2 Pro (Virtual)", Manufacturer::Apple, "M2 Pro"),
            ("Microsoft SQ2 @ 3.15 GHz", Manufacturer::Microsoft, ""),
            (
                "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Oryon(TM) CPU",
                Manufacturer::Qualcomm,
                "",
            ),
            ("Snapdragon (TM) 7c @ 2.40 GHz", Manufacturer::Qualcomm, ""),
            ("MediaTek Dimensity 9200", Manufacturer::MediaTek, ""),
            ("Samsung Exynos 2100", Manufacturer::Samsung, ""),
            ("Unknown CPU", Manufacturer::Unknown, ""),
        ];

        for (cpu_model, expected_manufacturer, expected_model) in tests {
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
        let tests = [
            (-42, "Some imaginary processor", PerformanceTier::Unknown),
            (0, "Some unknown processor", PerformanceTier::Unknown),
            (
                1,
                "Intel(R) Celeron(R) CPU          450  @ 2.20GHz",
                PerformanceTier::Low,
            ),
            (1, "AMD Athlon(tm) Processor 1640B", PerformanceTier::Low),
            (
                2,
                "AMD E2-9000e RADEON R2, 4 COMPUTE CORES 2C+2G",
                PerformanceTier::Low,
            ),
            (
                2,
                "AMD A4-9120C RADEON R4, 5 COMPUTE CORES 2C+3G",
                PerformanceTier::Low,
            ),
            (
                2,
                "AMD Athlon(tm) 64 X2 Dual Core Processor 5000+",
                PerformanceTier::Low,
            ),
            (
                4,
                "AMD Ryzen 3 3200U with Radeon Vega Mobile Gfx",
                PerformanceTier::Mid,
            ),
            (
                4,
                "AMD Ryzen 3 3100 4-Core Processor",
                PerformanceTier::High,
            ),
            (
                2,
                "Intel(R) Atom(TM) CPU Z520   @ 1.33GHz",
                PerformanceTier::Low,
            ),
            (
                2,
                "Intel(R) Celeron(R) CPU J3355 @ 2.00GHz",
                PerformanceTier::Low,
            ),
            (
                2,
                "Intel(R) Core(TM)2 Duo CPU     P8600  @ 2.40GHz",
                PerformanceTier::Low,
            ),
            (
                2,
                "Intel(R) Pentium(R) CPU        P6100  @ 2.00GHz",
                PerformanceTier::Low,
            ),
            (
                2,
                "Intel(R) Celeron(R) CPU B830 @ 1.80GHz",
                PerformanceTier::Low,
            ),
            (
                2,
                "Intel(R) Celeron(R) N4000 CPU @ 1.10GHz",
                PerformanceTier::Mid,
            ),
            (
                4,
                "Intel(R) Celeron(R) CPU  N3160  @ 1.60GHz",
                PerformanceTier::Low,
            ),
            (4, "Intel(R) N100", PerformanceTier::High),
            (4, "Intel(R) Atom(TM) x7425E", PerformanceTier::High),
            (
                8,
                "11th Gen Intel(R) Core(TM) i5-1135G7 @ 2.40GHz",
                PerformanceTier::High,
            ),
            (8, "Intel(R) Core(TM) Ultra 5 226V", PerformanceTier::Ultra),
            (16, "Intel(R) Core(TM) Ultra 7 155H", PerformanceTier::Ultra),
            (6, "Apple A18 Pro", PerformanceTier::High),
            (8, "Apple M1", PerformanceTier::Ultra),
            (
                10,
                "Snapdragon(R) X Plus - X1P64100 - Qualcomm(R) Oryon(TM) CPU",
                PerformanceTier::High,
            ),
            (
                12,
                "Snapdragon(R) X Elite - X1E78100 - Qualcomm(R) Oryon(TM) CPU",
                PerformanceTier::Ultra,
            ),
            (8, "MediaTek Dimensity 9200", PerformanceTier::High),
            (8, "Samsung Exynos 2100", PerformanceTier::High),
            (2, "Unknown CPU", PerformanceTier::Low),
            (4, "Unknown CPU", PerformanceTier::Mid),
            (8, "Unknown CPU", PerformanceTier::High),
            (16, "Unknown CPU", PerformanceTier::Ultra),
        ];

        for (cores, model, expected_tier) in tests {
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
}
