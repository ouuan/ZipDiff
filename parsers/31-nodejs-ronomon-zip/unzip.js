const ZIP = require('@ronomon/zip');
const { dirname } = require('path');
const { readFileSync, writeFileSync, mkdirSync } = require('fs');

const buffer = readFileSync(process.argv[2]);
try {
  const headers = ZIP.decode(buffer);
  process.chdir(process.argv[3]);
  for (const header of headers) {
    if (header.directory) {
      mkdirSync(header.fileName, { recursive: true });
    } else {
      mkdirSync(dirname(header.fileName), { recursive: true });
      const data = ZIP.inflate(header, buffer);
      writeFileSync(header.fileName, data);
    }
  }
} catch (error) {
  console.error(error.message);
  process.exit(1);
}
