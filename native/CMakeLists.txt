cmake_minimum_required(VERSION 3.1)
project(link-app-to-mic-example LANGUAGES CXX VERSION 1.0)

# --------------------------------------------------------------------------------------------------------
# Create library
# --------------------------------------------------------------------------------------------------------

file(GLOB src "main.cpp")

add_executable(${PROJECT_NAME} ${src})
target_compile_features(${PROJECT_NAME} PRIVATE cxx_std_17)
target_compile_options(${PROJECT_NAME} PRIVATE -Wall -Werror -Wextra -pedantic)
set_target_properties(${PROJECT_NAME} PROPERTIES CXX_STANDARD 17 CXX_EXTENSIONS OFF CXX_STANDARD_REQUIRED ON)

# --------------------------------------------------------------------------------------------------------
# Supply rohrkabel if example is built stand-alone
# --------------------------------------------------------------------------------------------------------

if(CMAKE_SOURCE_DIR STREQUAL CMAKE_CURRENT_SOURCE_DIR)
    include(FetchContent)
    FetchContent_Declare(rohrkabel GIT_REPOSITORY "https://github.com/Soundux/rohrkabel")
    FetchContent_MakeAvailable(rohrkabel)
endif()

# --------------------------------------------------------------------------------------------------------
# Link required libraries
# --------------------------------------------------------------------------------------------------------

target_link_libraries(${PROJECT_NAME} PRIVATE rohrkabel)
