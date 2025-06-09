uses
  Zipper;
var
  UnZipper: TUnZipper;
begin
  UnZipper := TUnZipper.Create;
  UnZipper.FileName := paramStr(1);
  UnZipper.OutputPath := paramStr(2);
  UnZipper.Examine;
  UnZipper.UnZipAllFiles;
  UnZipper.Free;
end.
