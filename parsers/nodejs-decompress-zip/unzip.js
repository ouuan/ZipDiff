const DecompressZip = require('decompress-zip');

const zip = new DecompressZip(process.argv[2]);

zip.on('error', (err) => {
  console.error(err);
  process.exit(1);
});

zip.extract({ path: process.argv[3] });
