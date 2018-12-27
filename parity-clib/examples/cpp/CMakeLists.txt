cmake_minimum_required(VERSION 3.5)
include(ExternalProject)

include_directories("${CMAKE_SOURCE_DIR}/../../../parity-clib")

add_executable(parity-example main.cpp)

ExternalProject_Add(
    libparity
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ""
    COMMAND cargo build -p parity-clib      # Note: use --release in a real project
    BINARY_DIR "${CMAKE_SOURCE_DIR}/../../../target"
    INSTALL_COMMAND ""
    LOG_BUILD ON)

add_dependencies(parity-example libparity)
target_link_libraries(parity-example "${CMAKE_SOURCE_DIR}/../../../target/debug/libparity.so")
