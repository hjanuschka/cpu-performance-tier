// Copyright 2025 The Chromium Authors
// Use of this source code is governed by a BSD-style license that can be
// found in the LICENSE file.

#ifndef CPU_PERFORMANCE_TIER_CPP_CPU_PERFORMANCE_TIER_H_
#define CPU_PERFORMANCE_TIER_CPP_CPU_PERFORMANCE_TIER_H_

#include <string>
#include <utility>
#include <vector>

namespace cpu_performance_tier {

enum class PerformanceTier {
  Unknown = 0,
  Low = 1,
  Mid = 2,
  High = 3,
  Ultra = 4,
};

enum class Manufacturer {
  Unknown,
  Amd,
  Apple,
  Intel,
  MediaTek,
  Microsoft,
  Qualcomm,
  Samsung,
};

struct PatternRule {
  Manufacturer manufacturer;
  int min_cores;
  int max_cores;
  PerformanceTier tier;
  std::vector<const char*> include;
  std::vector<const char*> exclude;
};

std::pair<Manufacturer, std::string> SplitCpuModel(const std::string& cpu_model);
PerformanceTier TierFromCores(int cores);
PerformanceTier TierFromCpuInfo(const std::string& cpu_model, int cores);

}  // namespace cpu_performance_tier

#endif  // CPU_PERFORMANCE_TIER_CPP_CPU_PERFORMANCE_TIER_H_
