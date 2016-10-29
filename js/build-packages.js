// Copyright 2015, 2016 Ethcore (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

const fs = require('fs-extra');

const mainPackage = require('./package.json');
const parityPackage = require('./parity.package.json');

const pkgs = {
  'parity.js': {
    json: parityPackage,
    files: ['parity.js']
  }
};

function createPackage (pkgName) {
  const pkg = pkgs[pkgName];
  const destDir = `.npmjs/${pkgName}`;
  const destJson = `${destDir}/package.json`;

  pkg.json.version = mainPackage.version;

  fs.ensureDir(destDir, (error) => {
    if (error) {
      console.error(`ensureDir ${destDir}`, error);
      process.exit(1);
    }

    fs.writeJson(destJson, pkg, (error) => {
      if (error) {
        console.error(`writeJson ${destJson}`, error);
        process.exit(2);
      }

      pkg.files.forEach((file) => {
        const srcFile = `.dist/build/${file}`;
        const destFile = `${destDir}/${file}`;

        fs.copy(srcFile, destFile, (error) => {
          if (error) {
            console.error(`copy ${srcFile} -> ${destFile}`, error);
            process.exit(3);
          }
        });
      });
    });
  });
}

Object.keys(pkgs).forEach(createPackage);
