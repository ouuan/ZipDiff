/* eslint-disable no-await-in-loop */

const { BlobReader, ZipReader, Uint8ArrayWriter } = require('@zip.js/zip.js');
const { dirname } = require('path');
const { openAsBlob } = require('fs');
const { mkdir, writeFile } = require('fs/promises');

(async () => {
  process.chdir(process.argv[3]);
  const file = await openAsBlob(process.argv[2]);
  const reader = new ZipReader(new BlobReader(file));
  for (const entry of await reader.getEntries()) {
    if (entry.directory) {
      await mkdir(entry.filename, { recursive: true });
    } else {
      const data = await entry.getData(new Uint8ArrayWriter());
      await mkdir(dirname(entry.filename), { recursive: true });
      await writeFile(entry.filename, data);
    }
  }
})();
