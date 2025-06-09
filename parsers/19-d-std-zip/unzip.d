import std.algorithm;
import std.file;
import std.path;
import std.zip;

void main(string[] args)
{
    auto zip = new ZipArchive(read(args[1]));
    chdir(args[2]);

    foreach (name, am; zip.directory)
    {
        if (am.name.endsWith('/')) {
            am.name.mkdirRecurse;
        } else {
            am.name.dirName.mkdirRecurse;
            zip.expand(am);
            write(am.name, am.expandedData);
        }
    }
}
