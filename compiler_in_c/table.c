#include "table.h"

#include <stdio.h>

#include "value.h"
#include <stdlib.h>
#include <string.h>

#include "memory.h"
#include "object.h"

#define TABLE_MAX_LOAD 0.75

void initTable(Table *table) {
  table->count = 0;
  table->capacity = 0;
  table->entries = NULL;
}

void freeTable(Table *table) {
  FREE_ARRAY(Entry, table->entries, table->capacity);
  initTable(table);
}

static Entry *findEntry(Entry *entries, const int capacity,
const ObjString *key) {
  uint32_t index = key->hash % capacity;
  Entry *tombstone = NULL;
  for (;;) {
    Entry *entry = &entries[index];
    if (entry->key == NULL) {
      if (IS_NIL(entry->value)) {
        return tombstone != NULL ? tombstone : entry;
      } else {
        if (tombstone == NULL)
          tombstone = entry;
      }

    } else if (entry->key == key) {
      return entry;
    }
    index = (index + 1) % capacity;
  }
}

bool tableGet(const Table *table, const ObjString *key, Value *value) {
  if (table->count == 0)
    return false;
  const Entry *entry = findEntry(table->entries, table->capacity, key);
  if (entry->key == NULL)
    return false;
  *value = entry->value;
  return true;
}

static void adjustCapacity(Table *table, const int capacity) {
  Entry *entries = ALLOCATE(Entry, capacity);
  for (int i = 0; i < capacity; i++) {
    entries[i].key = NULL;
    entries[i].value = NIL_VAL;
  }
  table->count = 0;
  for (int i = 0; i < table->capacity; i++) {
    const Entry *entry = &table->entries[i];
    if (entry->key == NULL)
      continue;
    Entry *dest = findEntry(entries, capacity, entry->key);
    dest->key = entry->key;
    dest->value = entry->value;
    table->count++;
  }
  FREE_ARRAY(Entry, table->entries, table->capacity);
  table->entries = entries;
  table->capacity = capacity;
}

bool tableSet(Table *table, ObjString *key, const Value value) {
  if (table->count + 1 > table->capacity * TABLE_MAX_LOAD) {
    const int capacity = GROW_CAPACITY(table->capacity);
    adjustCapacity(table, capacity);
  }
  Entry *entry = findEntry(table->entries, table->capacity, key);
  const bool isNewKey = entry->key == NULL;
  if (isNewKey && IS_NIL(entry->value))
    table->count++;
  entry->key = key;
  entry->value = value;
  return isNewKey;
}

bool tableDelete(const Table *table, const ObjString *key) {
  if (table->count == 0)
    return false;
  Entry *entry = findEntry(table->entries, table->capacity, key);
  if (entry->key == NULL)
    return false;

  entry->key = NULL;
  entry->value = BOOL_VAL(true);
  return true;
}

void tableAddAll(const Table *from, Table *to) {
  for (int i = 0; i < from->capacity; i++) {
    const Entry *entry = &from->entries[i];
    if (entry->key != NULL)
      tableSet(to, entry->key, entry->value);
  }
}

ObjString *tableFindString(const Table *table, const char *chars,
                           const int length, const uint32_t hash) {
  if (table->count == 0)
    return NULL;
  uint32_t index = hash % table->capacity;
  for (;;) {
    const Entry *entry = &table->entries[index];
    if (entry->key == NULL) {
      if (IS_NIL(entry->value)) {
        return NULL;
      }
    } else if (entry->key->length == length && entry->key->hash == hash &&
               memcmp(entry->key->chars, chars, length) == 0) {
      return entry->key;
    }
    index = index + 1 & table->capacity;
  }
}
