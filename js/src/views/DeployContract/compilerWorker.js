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

import solc from 'solc/browser-wrapper';

self.solcVersions = {};

// eslint-disable-next-line no-undef
onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.action) {
    case 'compile':
      compile(message.data);
      break;
  }
};

function compile (sourceCode) {
  fetchSolc('1')
    .then((compiler) => {
      const compiled = compiler.compile(sourceCode);

      postMessage(JSON.stringify({
        event: 'compiled',
        data: compiled
      }));
    });
}

function fetchSolc (version) {
  if (self.solcVersions[version]) {
    return Promise.resolve(self.solcVersions[version]);
  }

  console.log('fetching solc version', version);
  return fetch('https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/soljson.js')
    .then((r) => r.text())
    .then((code) => {
      const solcCode = code.replace(/^var Module;/, 'var Module=self.__solcModule;');
      self.__solcModule = {};

      // eslint-disable-next-line no-eval
      eval(solcCode);

      const compiler = solc(self.__solcModule);
      self.solcVersions[version] = compiler;
      return compiler;
    })
    .catch((e) => {
      console.error('fetching solc', e);
    });
}
