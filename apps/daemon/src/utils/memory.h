#ifndef MEMORY__H
#define MEMORY__H

#define scoped __attribute__((__cleanup__(free_scope)))

void free_scope(void *p);

#endif
