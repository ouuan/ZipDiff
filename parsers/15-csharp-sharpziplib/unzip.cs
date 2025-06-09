using ICSharpCode.SharpZipLib.Zip;

using (ZipFile zipFile = new ZipFile(args[0]))
{
    if (!zipFile.TestArchive(true))
    {
        System.Environment.Exit(1);
    }
}

new FastZip().ExtractZip(args[0], args[1], null);
