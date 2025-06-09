using SharpCompress.Archives.Zip;
using SharpCompress.Common;
using SharpCompress.Readers;

using (var archive = ZipArchive.Open(args[0]))
{
    var opt = new ExtractionOptions()
    {
        ExtractFullPath = true,
        Overwrite = true
    };
    archive.ExtractAllEntries().WriteAllToDirectory(args[1], opt);
}
