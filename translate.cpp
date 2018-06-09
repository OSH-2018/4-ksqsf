// Reference: pagemap.txt.

#include <cstdio>
#include <cstdlib>
#include <cstdint>
using namespace std;

#define PAGE_SIZE 4096
#define IF_SWAPPED(x) ((x>>62) & 1)
#define IF_PRESENT(x) ((x>>63) & 1)
#define PFN(x) ((x) & ((1ULL<<54)-1))

void do_translate(pid_t pid, size_t virt)
{
    char path[256];
    FILE *f;
    uint64_t entry;

    sprintf(path, "/proc/%d/pagemap", pid);
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
        printf("pfn=%lx, phys_addr=%lX\n", pfn, pfn*PAGE_SIZE + virt%PAGE_SIZE);
    }
    fclose(f);
}

int main(int argc, char *argv[])
{
    if (argc != 3) {
        printf("translate PID VIRT\n");
        return 0;
    }

    pid_t pid = atoi(argv[1]);
    size_t virt = strtoull(argv[2], NULL, 16);
    do_translate(pid, virt);
    return 0;
}
