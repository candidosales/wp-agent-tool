// Test fixture: stands in for a PHP worker binary. `-v` prints the version
// given at compile time (-DVER=...); otherwise it idles like a worker.
#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main(int argc, char **argv) {
    if (argc > 1 && strcmp(argv[1], "-v") == 0) {
        printf("PHP %s (litespeed) (built: Oct 21 2026)\n", VER);
        return 0;
    }
    pause();
    return 0;
}
