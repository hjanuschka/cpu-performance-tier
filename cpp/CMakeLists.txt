cmake_minimum_required(VERSION 3.16)
project(cpu_performance_tier_cpp LANGUAGES CXX)

add_library(cpu_performance_tier
  cpu_performance_tier.cc
)
target_include_directories(cpu_performance_tier PUBLIC ${CMAKE_CURRENT_SOURCE_DIR})
target_compile_features(cpu_performance_tier PUBLIC cxx_std_17)

add_executable(cpu_performance_tier_tests
  generated_tests.cc
)
target_link_libraries(cpu_performance_tier_tests PRIVATE cpu_performance_tier)

include(CTest)
add_test(NAME cpu_performance_tier_tests COMMAND cpu_performance_tier_tests)
