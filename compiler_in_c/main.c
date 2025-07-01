#include "chunk.h"
#include "common.h"
#include "debug.h"

// original
int main(int argc, const char *argv[]) {
  Chunk chunk;
  initChunk(&chunk);
  int constant = addConstant(&chunk, 1.2);
  writeChunk(&chunk, OP_CONSTANT, 1);
  writeChunk(&chunk, constant, 1);
  writeChunk(&chunk, OP_RETURN, 1);
  disassembleChunk(&chunk, "test chunk");
  freeChunk(&chunk);
  return 0;
}

// modified
// int main(int argc, const char *argv[]) {
//     Chunk chunk;
//     initChunk(&chunk);
//     for (int i=0; i<300; i++) {
//         writeConstant(&chunk, i, i);
//     }
//     disassembleChunk(&chunk, "test chunk");
//     freeChunk(&chunk);
//     return 0;
// }
