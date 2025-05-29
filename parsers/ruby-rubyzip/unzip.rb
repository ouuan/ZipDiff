require 'zip'

Zip.on_exists_proc = true
Dir.chdir(ARGV[1])

Zip::File.open(ARGV[0]) do |zip_file|
  zip_file.each do |entry|
    entry_path = File.join(Dir.pwd, entry.name)
    FileUtils.mkdir_p(File.dirname(entry_path))
    entry.extract
  end
end
