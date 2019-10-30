cmake_minimum_required(VERSION 3.5)
include(ExternalProject)
include_directories("${CMAKE_SOURCE_DIR}/../..")
set (CMAKE_CXX_STANDARD 17) # Enforce C++17
set (CMAKE_CXX_STANDARD_REQUIRED ON) # Enforce C++17
set (CMAKE_CXX_EXTENSIONS OFF) # Enforce C++17
set (CMAKE_EXPORT_COMPILE_COMMANDS ON) # generate a compilation database
add_executable(parity-example main.cpp)
add_compile_options("-Wall -Wextra -Werror")

ExternalProject_Add(
    libparity
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND ""    # Note: use --release in a real project
    COMMAND cargo build -p parity-clib --verbose --color=always # Note: use --release in a real project
    BINARY_DIR "${CMAKE_SOURCE_DIR}/../../../target"
    INSTALL_COMMAND ""
    LOG_BUILD ON)

add_dependencies(parity-example libparity)
target_link_libraries(parity-example "${CMAKE_SOURCE_DIR}/../../../target/release/${CMAKE_SHARED_LIBRARY_PREFIX}parity${CMAKE_SHARED_LIBRARY_SUFFIX}")
