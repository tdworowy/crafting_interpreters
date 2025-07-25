#include "memory.h"
#include <stdlib.h>

#include "object.h"
#include "value.h"
#include "vm.h"

void *reallocate(void *pointer, size_t oldSize, const size_t newSize) {
  if (newSize == 0) {
    free(pointer);
    return NULL;
  }

  void *result = realloc(pointer, newSize);
  if (result == NULL)
    exit(1);
  return result;
}

void freeObject(Obj *object) {
  switch (object->type) {
  case OBJ_STRING: {
    const ObjString *string = (ObjString *)object;
    FREE_ARRAY(char, string->chars, string->length + 1);
    FREE(ObjString, object);
    break;
  }
  }
}

void freeObjects() {
  Obj *object = vm.objects;
  while (object != NULL) {
    Obj *next = object->next;
    freeObject(object);
    object = next;
  }
}