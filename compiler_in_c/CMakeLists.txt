cmake_minimum_required(VERSION 3.31)
project(compiler_in_c C)

set(CMAKE_C_STANDARD 11)

include_directories(.)

add_executable(compiler_in_c
        chunk.c
        chunk.h
        common.h
        memory.c
        memory.h
        debug.c
        debug.h
        main.c)
