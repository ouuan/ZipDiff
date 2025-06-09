const StreamZip = require('node-stream-zip');

const zip = new StreamZip.async({ file: process.argv[2]});
zip.extract(null, process.argv[3]);
