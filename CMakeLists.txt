cmake_minimum_required(VERSION 3.7)
project(gameboy)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

# Prevent in-source builds
if(${PROJECT_NAME}_SOURCE_DIR STREQUAL ${PROJECT_NAME}_BINARY_DIR)
    message(FATAL_ERROR "In-source builds are not allowed.")
endif(${PROJECT_NAME}_SOURCE_DIR STREQUAL ${PROJECT_NAME}_BINARY_DIR)

enable_testing()

add_subdirectory(core)
add_subdirectory(emulator)
