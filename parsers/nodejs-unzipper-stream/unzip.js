const { createReadStream } = require('fs');
const { Extract } = require('unzipper');

const extract = Extract({ path: process.argv[3] });
createReadStream(process.argv[2]).pipe(extract);
extract.on('error', (error) => {
  console.error(error);
  process.exit(1);
});
