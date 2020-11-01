#ifndef __KNS_SYS_MMAN_H
#define __KNS_SYS_MMAN_H

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
#include <sys/types.h>

#ifdef __cplusplus
extern "C" {
#endif

#define MAP_FAILED ((void *)-1)

#define PROT_READ 0x1
#define PROT_WRITE 0x2
#define PROT_EXEC 0x4
#define PROT_SEM 0x8
#define PROT_NONE 0x0

#define MAP_32BIT 0
#define MAP_HUGE_2MB 0
#define MAP_HUGE_1GB 0
#define MAP_UNINITIALIZED 0

#define MAP_SHARED 0x01
#define MAP_PRIVATE 0x02
#define MAP_SHARED_VALIDATE 0x03
#define MAP_FIXED 0x10
#define MAP_ANONYMOUS 0x20

#define MAP_POPULATE 0x008000
#define MAP_NONBLOCK 0x010000
#define MAP_STACK 0x020000
#define MAP_HUGETLB 0x040000
#define MAP_SYNC 0x080000
#define MAP_FIXED_NOREPLACE 0x100000

#define MADV_NORMAL 0
#define MADV_RANDOM 1
#define MADV_SEQUENTIAL 2
#define MADV_WILLNEED 3
#define MADV_DONTNEED 4

#define MADV_FREE 8
#define MADV_REMOVE 9
#define MADV_DONTFORK 10
#define MADV_DOFORK 11
#define MADV_HWPOISON 100
#define MADV_SOFT_OFFLINE 101

#define MADV_MERGEABLE 12
#define MADV_UNMERGEABLE 13

#define MADV_HUGEPAGE 14
#define MADV_NOHUGEPAGE 15

#define MADV_DONTDUMP 16
#define MADV_DODUMP 17

#define MADV_WIPEONFORK 18
#define MADV_KEEPONFORK 19

#define MADV_COLD 20
#define MADV_PAGEOUT 21

#define POSIX_MADV_NORMAL MADV_NORMAL
#define POSIX_MADV_SEQUENTIAL MADV_SEQUENTIAL
#define POSIX_MADV_RANDOM MADV_RANDOM
#define POSIX_MADV_WILLNEED MADV_WILLNEED
#define POSIX_MADV_DONTNEED MADV_DONTNEED

extern void *mmap(void *addr, size_t length, int prot, int flags, int fd,
                  off_t offset);
extern int munmap(void *addr, size_t length);
extern int madvise(void *addr, size_t length, int advice);
extern int posix_madvise(void *addr, size_t length, int advice);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
