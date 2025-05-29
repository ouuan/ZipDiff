const { loadAsync } = require('jszip');
const { dirname } = require('path');
const { readFile, mkdir, writeFile } = require('fs/promises');

(async () => {
  const file = await readFile(process.argv[2]);
  const zip = await loadAsync(file);

  process.chdir(process.argv[3]);

  for (const entry of Object.values(zip.files)) {
    if (entry.dir) {
      await mkdir(entry.name, { recursive: true });
    } else {
      await mkdir(dirname(entry.name), { recursive: true });
      const content = await entry.async('nodebuffer');
      await writeFile(entry.name, content);
    }
  }
})();
