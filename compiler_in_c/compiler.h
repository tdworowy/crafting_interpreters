#ifndef clox_compiler_h
#define clox_compiler_h
#include "object.h"
ObjFunction *compile(char *source);
void markCompilerRoots();
#endif