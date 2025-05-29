#include <Poco/Delegate.h>
#include <Poco/Zip/Decompress.h>

#include <fstream>
#include <iostream>

void onDecompressError(const void *pSender,
                       std::pair<const Poco::Zip::ZipLocalFileHeader, const std::string> &info)
{
    const Poco::Zip::ZipLocalFileHeader &header = info.first;
    const std::string &errorMsg = info.second;

    std::cerr << "Error decompressing file: " << header.getFileName() << std::endl;
    std::cerr << "Error message: " << errorMsg << std::endl;

    std::exit(1);
}

int main(int argc, char **argv)
{
    std::ifstream inp(argv[1], std::ios::binary);
    Poco::Zip::Decompress dec(inp, Poco::Path(argv[2]));
    dec.EError += Poco::delegate(&onDecompressError);
    dec.decompressAllFiles();
}
