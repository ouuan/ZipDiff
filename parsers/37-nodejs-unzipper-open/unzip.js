const { Open } = require('unzipper');

(async () => {
  try {
    const d = await Open.file(process.argv[2]);
    await d.extract({ path: process.argv[3] });
  } catch (err) {
    console.error(err);
    process.exit(1);
  }
})();
