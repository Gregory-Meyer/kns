#include <stdio.h>

int main(int argc, char *argv[argc]) {
  for (int i = 1; i < argc; ++i) {
    FILE *const f = fopen(argv[i], "r");

    if (!f) {
      continue;
    }

    char buf[512];

    while (fgets(buf, (int)sizeof(buf), f)) {
      fputs(buf, stdout);
    }

    fclose(f);
  }
}
