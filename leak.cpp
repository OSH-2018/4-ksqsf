// This source file contains work from IAIK/meltdown.
// Specifically, how to prevent data from being swapped.

#include <bits/stdc++.h>
#include <unistd.h>
#include <sched.h>
using namespace std;

#define PAGE_SIZE 4096
#define IF_SWAPPED(x) ((x>>62) & 1)
#define IF_PRESENT(x) ((x>>63) & 1)
#define PFN(x) ((x) & ((1ULL<<54)-1))

size_t do_translate(pid_t pid, size_t virt)
{
    char path[256];
    FILE *f;
    uint64_t entry;

    if (pid)
        sprintf(path, "/proc/%d/pagemap", pid);
    else
        strcpy(path, "/proc/self/pagemap");
    f = fopen(path, "rb");
    fseek(f, virt / PAGE_SIZE * sizeof(uint64_t), SEEK_SET);
    fread(&entry, sizeof(entry), 1, f);

    if (IF_SWAPPED(entry)) {
        printf("swapped\n");
    }
    else if (!IF_PRESENT(entry)) {
        printf("!present\n");
    }
    else {
        uint64_t pfn = PFN(entry);
        return pfn*PAGE_SIZE + virt%PAGE_SIZE;
    }
    fclose(f);
    return 0;
}

const char *dokidoki = "\"There's a little devil inside all of us.\" Beneath their manufactured perception - their artificial reality - is a writhing, twisted mess of dread.\n";

int main()
{
    printf("I'm %d  My secret is at %lX\n", getpid(), reinterpret_cast<uint64_t>(dokidoki));
    printf("phys = %lx\n", do_translate(0, (size_t)dokidoki));

    while (true) {
        volatile size_t hash = 0;
        for (unsigned i = 0; i < strlen(dokidoki); ++i) {
            hash += dokidoki[i];
        }
        sched_yield();
    }
    return 0;
}
