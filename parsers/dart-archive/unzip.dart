import 'package:archive/archive_io.dart';

void main(List<String> args) {
  final archive = ZipDecoder().decodeBuffer(InputFileStream(args[0]));
  extractArchiveToDisk(archive, args[1]);
}
