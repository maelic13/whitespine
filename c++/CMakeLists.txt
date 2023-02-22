cmake_minimum_required(VERSION 3.24)
project(Engine
        VERSION 0.1)

set(CMAKE_CXX_STANDARD 23)

add_executable(engine main.cpp UciProtocol.cpp UciProtocol.h SearchOptions.cpp SearchOptions.h
        Engine.cpp Engine.h Engine.h Constants.h)
