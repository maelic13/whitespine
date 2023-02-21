cmake_minimum_required(VERSION 3.24)
project(engine)

set(CMAKE_CXX_STANDARD 23)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

add_executable(engine main.cpp UciProtocol.cpp UciProtocol.h SearchOptions.cpp SearchOptions.h Engine.cpp Engine.h Engine.h Constants.h)
