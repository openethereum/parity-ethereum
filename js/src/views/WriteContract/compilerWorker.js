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

function compile (data) {
  const { sourcecode, build } = data;

  fetchSolc(build.path)
    .then((compiler) => {
      const compiled = compiler.compile(sourcecode);

      postMessage(JSON.stringify({
        event: 'compiled',
        data: compiled
      }));
    });
}

function fetchSolc (path) {
  if (self.solcVersions[path]) {
    return Promise.resolve(self.solcVersions[path]);
  }

  const URL = `https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/${path}`;
  console.log(`fetching solc version ${path} at ${URL}`);

  return fetch(URL)
    .then((r) => r.text())
    .then((code) => {
      const solcCode = code.replace(/^var Module;/, 'var Module=self.__solcModule;');
      self.__solcModule = {};

      // eslint-disable-next-line no-eval
      eval(solcCode);

      const compiler = solc(self.__solcModule);
      self.solcVersions[path] = compiler;
      return compiler;
    })
    .catch((e) => {
      console.error('fetching solc', e);
    });
}
