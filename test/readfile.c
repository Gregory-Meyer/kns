#include <stdio.h>

int main(int argc, char *argv[argc]) {
  if (argc < 2) {
    return 1;
  }

  FILE *const f = fopen(argv[1], "r");

  char buf[16] = {0};
  if (fgets(buf, (int)sizeof(buf), f)) {
    fputs(buf, stdout);
  }

  fclose(f);
}
