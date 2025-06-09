require 'zip'

Zip.on_exists_proc = true
Dir.chdir(ARGV[1])

stream = Zip::InputStream.new(ARGV[0])
while entry = stream.get_next_entry
  entry_path = File.join(Dir.pwd, entry.name)
  FileUtils.mkdir_p(File.dirname(entry_path))
  entry.extract
end
