#include "zip.h"

int main(int args, char **argv)
{
    return zip_extract(argv[1], argv[2], NULL, NULL);
}
