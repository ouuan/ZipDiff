from zipfile import ZipFile
from sys import argv

zip = ZipFile(argv[1], 'r')
error_file = zip.testzip()
if error_file is None:
    zip.extractall(argv[2])
else:
    print(f"Error in file {error_file}")
    exit(1)
