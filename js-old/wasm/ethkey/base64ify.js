const fs = require('fs');

const file = fs.readFileSync('./ethkey.opt.wasm', { encoding: 'base64' });

fs.writeFileSync('../../src/api/local/ethkey/ethkey.wasm.js', `module.exports = new Buffer('${file}', 'base64');\n`);
