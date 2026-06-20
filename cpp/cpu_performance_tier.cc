// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#include "cpu_performance_tier.h"

#include <algorithm>
#include <cctype>
#include <regex>
#include <string>

#include "generated_rules.inc"

namespace cpu_performance_tier {
namespace {

std::regex MakeRegex(const std::string& pattern) {
  std::regex::flag_type flags = std::regex::ECMAScript;
  std::string body = pattern;
  if (body.rfind("(?i)", 0) == 0) {
    flags |= std::regex::icase;
    body = body.substr(4);
  }
  return std::regex(body, flags);
}

bool Search(const std::string& text, const std::string& pattern) {
  return std::regex_search(text, MakeRegex(pattern));
}

void Replace(std::string* text,
             const std::string& pattern,
             const std::string& replacement) {
  *text = std::regex_replace(*text, MakeRegex(pattern), replacement);
}

void ReplaceFirst(std::string* text,
                  const std::string& pattern,
                  const std::string& replacement) {
  *text = std::regex_replace(*text, MakeRegex(pattern), replacement,
                             std::regex_constants::format_first_only);
}

bool IsWs(unsigned char c) {
  return std::isspace(c) || (c >= 0x1c && c <= 0x1f);
}

void TrimAndCollapseWhitespace(std::string* text) {
  std::string out;
  bool pending_space = false;
  bool seen_non_space = false;
  for (unsigned char c : *text) {
    if (IsWs(c)) {
      if (seen_non_space) {
        pending_space = true;
      }
      continue;
    }
    if (pending_space && !out.empty()) {
      out.push_back(' ');
    }
    out.push_back(static_cast<char>(c));
    seen_non_space = true;
    pending_space = false;
  }
  *text = out;
}

Manufacturer GetManufacturer(const std::string& cpu_model) {
  if (Search(cpu_model, R"((?i)\bAMD\b)")) {
    return Manufacturer::Amd;
  } else if (Search(cpu_model, R"((?i)\bApple\b)")) {
    return Manufacturer::Apple;
  } else if (Search(cpu_model, R"((?i)\b(Intel|Celeron|Pentium)\b)")) {
    return Manufacturer::Intel;
  } else if (Search(cpu_model, R"((?i)\bMediaTek\b)")) {
    return Manufacturer::MediaTek;
  } else if (Search(cpu_model, R"((?i)\bMicrosoft\b)")) {
    return Manufacturer::Microsoft;
  } else if (Search(cpu_model, R"((?i)\b(Qualcomm|Snapdragon)\b)")) {
    return Manufacturer::Qualcomm;
  } else if (Search(cpu_model, R"((?i)\bSamsung\b)")) {
    return Manufacturer::Samsung;
  }
  return Manufacturer::Unknown;
}

}  // namespace

std::pair<Manufacturer, std::string> SplitCpuModel(const std::string& cpu_model) {
  std::string text = cpu_model;

  TrimAndCollapseWhitespace(&text);

  Manufacturer manufacturer = GetManufacturer(text);

  Replace(&text, R"(\([^)]*\))", " ");
  Replace(&text, R"(\$|®|™)", " ");
  Replace(&text, R"((?i)@( )?\d[.,]\d+([~-]\d[.,]\d+)?( )?GHz\b)", "");
  Replace(&text, R"((?i)\b\d[.,]\d+([~-]\d[.,]\d+)?( )?GHz\b)", "");

  TrimAndCollapseWhitespace(&text);

  Replace(&text, R"((^| )?[@~\-,.]$)", "");

  Replace(&text, R"((?i)\bCPU\b)", "");
  Replace(&text, R"((?i)\bMobile\b)", "");
  Replace(&text, R"((?i)\bProcessor\b)", "");
  Replace(&text, R"((?i)\bSilicon\b)", "");
  Replace(&text, R"((?i)\bSOC\b)", "");
  Replace(&text, R"((?i)\bTechnology\b)", "");

  TrimAndCollapseWhitespace(&text);

  switch (manufacturer) {
    case Manufacturer::Amd:
      ReplaceFirst(&text, R"((?i).*?\bAMD\b)", "");
      TrimAndCollapseWhitespace(&text);
      Replace(&text, R"((?i)\bFX -)", "FX-");
      Replace(&text, R"((?i)\+( )?(AMD )?Radeon.*)", "");
      Replace(&text, R"((?i)\b(RADEON )?R\d+, \d+ COMPUTE CORES \d+C\+\d+G\b)", "");
      Replace(&text, R"((?i)\bwith (AMD )?Radeon.*)", "");
      Replace(&text, R"((?i)\bw/( )?(AMD )?Radeon.*)", "");
      Replace(&text, R"((?i)\bRadeon.*)", "");
      Replace(&text, R"((?i)\b\w+( |-)Core\b)", "");
      Replace(&text, R"((?i)\b\d+-Core(s)?\b)", "");
      Replace(&text, R"((?i)\bAPU\b)", "");
      Replace(&text, R"((?i)\bCreator Edition\b)", "");
      Replace(&text, R"((?i)\bDesktop Kit\b)", "");
      Replace(&text, R"((?i)\b(3250C) 15W\b)", "$1");
      break;
    case Manufacturer::Apple:
      ReplaceFirst(&text, R"((?i).*?\bApple\b)", "");
      break;
    case Manufacturer::Intel:
      ReplaceFirst(&text, R"((?i).*?\bIntel\b)", "");
      TrimAndCollapseWhitespace(&text);
      Replace(&text, R"((?i)\b(Core)(2)\b)", "$1 $2");
      Replace(&text, R"((?i)\b(Core i\d+)( )?-( )?)", "$1-");
      Replace(&text, R"((?i)\b(Core i\d+) (M) (\d+)\b)", "$1-$3$2");
      Replace(&text, R"((?i)\b(Core i\d+) ([LQU]) (\d+)\b)", "$1-$3$2M");
      Replace(&text, R"((?i)\b(Core i\d+) (\d+)\b)", "$1-$2");
      Replace(&text, R"((?i)\b(Celeron|Pentium) Dual(-Core)?\b)", "$1");
      Replace(&text, R"(\b0+$)", "");
      break;
    default:
      return {manufacturer, ""};
  }

  TrimAndCollapseWhitespace(&text);
  return {manufacturer, text};
}

PerformanceTier TierFromCores(int cores) {
  if (cores >= 1 && cores <= 2) {
    return PerformanceTier::Low;
  } else if (cores >= 3 && cores <= 4) {
    return PerformanceTier::Mid;
  } else if (cores >= 5 && cores <= 12) {
    return PerformanceTier::High;
  } else if (cores >= 13) {
    return PerformanceTier::Ultra;
  }
  return PerformanceTier::Unknown;
}

PerformanceTier TierFromCpuInfo(const std::string& cpu_model, int cores) {
  if (cores <= 0) {
    return PerformanceTier::Unknown;
  }
  if (cores <= 1) {
    return PerformanceTier::Low;
  }

  auto [manufacturer, model] = SplitCpuModel(cpu_model);

  for (size_t i = 0; i < generated::kPatternRuleCount; ++i) {
    const PatternRule& rule = generated::kPatternRules[i];
    if (rule.manufacturer != manufacturer || cores < rule.min_cores || cores > rule.max_cores) {
      continue;
    }
    bool included = false;
    for (const char* pattern : rule.include) {
      if (Search(model, pattern)) {
        included = true;
        break;
      }
    }
    if (!included) {
      continue;
    }
    bool excluded = false;
    for (const char* pattern : rule.exclude) {
      if (Search(model, pattern)) {
        excluded = true;
        break;
      }
    }
    if (!excluded) {
      return rule.tier;
    }
  }

  if (cores <= 4) {
    if (manufacturer != Manufacturer::Amd && manufacturer != Manufacturer::Intel && cores <= 2) {
      return PerformanceTier::Low;
    }
    return PerformanceTier::Mid;
  }

  if (cores <= 10) {
    return PerformanceTier::High;
  }

  return PerformanceTier::Ultra;
}

}  // namespace cpu_performance_tier
