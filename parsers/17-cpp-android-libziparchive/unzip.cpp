#include <fcntl.h>
#include <sys/stat.h>
#include <ziparchive/zip_archive.h>

#include <cstdio>
#include <filesystem>
#include <string>

int main(int argc, char **argv)
{
    ZipArchiveHandle archive;
    if (OpenArchive(argv[1], &archive) < 0)
    {
        fputs("Failed to open ZIP archive", stderr);
        return 1;
    }

    void *cookie;
    if (StartIteration(archive, &cookie) < 0)
    {
        fputs("Failed to iterate over ZIP entries", stderr);
        return 2;
    }

    const auto targetDir = std::filesystem::path(argv[2]);

    while (true)
    {
        ZipEntry entry;
        std::string name;

        const int status = Next(cookie, &entry, &name);
        if (status == -1)
            break;
        if (status < -1)
        {
            fputs("Failed to get next entry", stderr);
            return 3;
        }

        const auto target = targetDir / name;

        if (name.back() == '/')
        {
            std::filesystem::create_directories(target);
        }
        else
        {
            std::filesystem::create_directories(target.parent_path());

            int fd = creat(target.c_str(), 0644);
            if (fd < 0)
            {
                fputs("Failed to open output file", stderr);
                return 4;
            }

            if (ExtractEntryToFile(archive, &entry, fd) < 0)
            {
                fputs("Failed to extract to output file", stderr);
                return 5;
            }
        }
    }
}
