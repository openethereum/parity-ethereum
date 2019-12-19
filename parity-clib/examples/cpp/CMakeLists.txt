cmake_minimum_required(VERSION 3.5)
project("Parity Ethereum C API example")
include(ExternalProject)
include_directories("${CMAKE_SOURCE_DIR}/../..")
set (CMAKE_C_COMPILER "clang")
set (CMAKE_CXX_COMPILER "clang++")
set (CMAKE_CXX_STANDARD 17) # Enforce C++17
set (CMAKE_CXX_STANDARD_REQUIRED ON) # Enforce C++17
set (CMAKE_CXX_EXTENSIONS OFF) # Enforce C++17
set (CMAKE_EXPORT_COMPILE_COMMANDS ON) # generate a compilation database
add_executable(parity-example main.cpp)
add_compile_options("-Wall -Wextra -Werror")
set (parity_build_flavor "debug") # would use --release for a real project
if(parity_build_flavor STREQUAL "debug")
   set (parity_rust_build_option "")
elseif(parity_build_flavor STREQUAL "release")
   set (parity_rust_build_option "--release")
else()
   message(FATAL_ERROR "Invalid parity_rust_build_option")
endif()
set(parity_shared_library_path "${CMAKE_SOURCE_DIR}/../../../target/${parity_build_flavor}/${CMAKE_SHARED_LIBRARY_PREFIX}parity${CMAKE_SHARED_LIBRARY_SUFFIX}")
ExternalProject_Add(
    libparity
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build -p parity-clib ${parity_rust_build_option} --verbose --color=always
    BINARY_DIR "${CMAKE_SOURCE_DIR}/../../../target"
    BUILD_BYPRODUCTS "${parity_shared_library_path}"
    USES_TERMINAL_BUILD ON
    INSTALL_COMMAND ""
    BUILD_ALWAYS ON
    LOG_BUILD ON
    VERBATIM)

add_dependencies(parity-example libparity)
target_link_libraries(parity-example "${parity_shared_library_path}")
