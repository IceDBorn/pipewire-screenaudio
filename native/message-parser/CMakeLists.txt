cmake_minimum_required(VERSION 3.2.2)
project(message-parser LANGUAGES CXX VERSION 1.0)

file(GLOB src "message-parser.c")

add_executable(${PROJECT_NAME} ${src})
