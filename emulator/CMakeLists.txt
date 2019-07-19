# Executable
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOUIC ON)
set(CMAKE_AUTORCC ON)
# set(CMAKE_INCLUDE_CURRENT_DIR ON)

add_executable(emulator WIN32 # MACOSX_BUNDLE
    core.cpp
    core.hpp
    main_window.cpp
    main_window.hpp
    main.cpp
    # vram_window.hpp
)

# Use c++ 14
set_target_properties(emulator PROPERTIES
    CXX_STANDARD 14
    CXX_STANDARD_REQUIRED YES
    CXX_EXTENSIONS NO
)

# Set warning levels
if (MSVC)
    ## remove default warning level from CMAKE_CXX_FLAGS
    string(REGEX REPLACE "/W[0-4]" "" CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")

    target_compile_options(emulator PRIVATE /W4 /WX /wd4100)
else(MSVC)
    if (CMAKE_CXX_COMPILER_ID MATCHES "Clang")
        target_compile_options(emulator PRIVATE -Wextra-semi -Wnull-dereference -Wunused-exception-parameter)
    endif(CMAKE_CXX_COMPILER_ID MATCHES "Clang")

    target_compile_options(emulator PRIVATE -Wall -Wextra -Wconversion -Wold-style-cast -Wpedantic -Werror -Wno-unused-parameter)
endif(MSVC)

# Core library
get_target_property(CORE_DIR core LOCATION)

add_dependencies(emulator core)
target_link_libraries(emulator ${CORE_DIR}/libcore.dylib)

# Dependencies

## QtWidgets
find_package(Qt5 COMPONENTS Widgets REQUIRED)
target_link_libraries(emulator Qt5::Widgets)
