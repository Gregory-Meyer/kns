// Copyright (C) 2020 Gregory Meyer
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#include <stdlib.h>

#include "rpmalloc.h"

void *malloc(size_t size) { return rpmalloc(size); }

void free(void *ptr) { rpfree(ptr); }

void *calloc(size_t nmemb, size_t size) { return rpcalloc(nmemb, size); }

void *realloc(void *ptr, size_t size) { return rprealloc(ptr, size); }

int posix_memalign(void **memptr, size_t alignment, size_t size) {
  return rpposix_memalign(memptr, alignment, size);
}

void *aligned_alloc(size_t alignment, size_t size) {
  return rpaligned_alloc(alignment, size);
}

void *__KNS_aligned_calloc(size_t alignment, size_t nmemb, size_t size) {
  return rpaligned_calloc(alignment, nmemb, size);
}

void *__KNS_aligned_realloc(void *ptr, size_t alignment, size_t size,
                            size_t oldsize) {
  return rpaligned_realloc(ptr, alignment, size, oldsize, 0);
}
