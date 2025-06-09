using Ionic.Zip;

if (ZipFile.CheckZip(args[0]))
{
    using (ZipFile zip = ZipFile.Read(args[0]))
    {
        zip.ExtractAll(args[1], ExtractExistingFileAction.OverwriteSilently);
    }
}
else
{
    System.Environment.Exit(1);
}
