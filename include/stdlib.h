#ifndef __KNS_STDLIB_H
#define __KNS_STDLIB_H

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

#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define EXIT_SUCCESS ((int)0)
#define EXIT_FAILURE ((int)1)

extern long strtol(const char *nptr, char **endptr, int base);

extern void *malloc(size_t size);
extern void free(void *ptr);
extern void *calloc(size_t nmemb, size_t size);
extern void *realloc(void *ptr, size_t size);

extern int posix_memalign(void **memptr, size_t alignment, size_t size);
extern void *aligned_alloc(size_t alignment, size_t size);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
