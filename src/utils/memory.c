#include <stdlib.h>

#include "memory.h"

void
free_scope(void *p)
{
    free(*((void **)p));
}
