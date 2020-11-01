# libkns

x86_64-linux-gnu libc.

## Features

* `_start`
* `stdin`, `stdout`, `stderr`, `fputs`, and `fgets`

## Future Features

* `pthread_create`, `pthread_join`, `pthread_detach`
* `io_uring`-backed implementations of `read`, `write`, etc

## Compiling

```bash
cargo build
clang test/cat.c -c -o cat.o -nostdinc -nostdlib -nodefaultlibs -isysteminclude
clang target/debug/build/kns-*/out/crt0.o cat.o -lkns -o cat -nostdlib -nodefaultlibs -Ltarget/debug
./cat README.md
```

## Name

As in kuchh nahin se achchha (but only just barely).

## License

libkns is licensed under the AGPL-3.0-or-later license. x86_64 assembly
implementations of memcpy, memmove, and memset from [musl libc] are licensed
under the MIT license. [rpmalloc] is unlicensed and released to the public
domain.

[musl libc]: https://musl.libc.org/
[rpmalloc]: https://github.com/mjansson/rpmalloc
