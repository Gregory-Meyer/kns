#ifndef __KNS_STDIO_H
#define __KNS_STDIO_H

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

#ifdef __cplusplus
extern "C" {
#endif

#define stdin (__KNS_stdin())
#define stdout (__KNS_stdout())
#define stderr (__KNS_stderr())

#define EOF ((int)-1)

typedef struct FILE FILE;

extern FILE *fopen(const char *pathname, const char *mode);
extern int fclose(FILE *stream);

extern char *fgets(char *s, int size, FILE *stream);
extern int fputs(const char *s, FILE *stream);

extern FILE *__KNS_stdin(void);
extern FILE *__KNS_stdout(void);
extern FILE *__KNS_stderr(void);

#ifdef __cplusplus
} // extern "C"
#endif

#endif
