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
import { isWebUri } from 'valid-url';

self.solcVersions = {};
self.files = {};
self.lastCompile = {
  sourcecode: '',
  result: '',
  version: ''
};

// eslint-disable-next-line no-undef
onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.action) {
    case 'compile':
      compile(message.data);
      break;
    case 'load':
      load(message.data);
      break;
    case 'setFiles':
      setFiles(message.data);
      break;
    case 'close':
      close();
      break;
  }
};

function setFiles (files) {
  const prevFiles = self.files;
  const nextFiles = files.reduce((obj, file) => {
    obj[file.name] = file.sourcecode;
    return obj;
  }, {});

  self.files = {
    ...prevFiles,
    ...nextFiles
  };
}

function findImports (path) {
  if (self.files[path]) {
    if (self.files[path].error) {
      return { error: self.files[path].error };
    }

    return { contents: self.files[path] };
  }

  if (isWebUri(path)) {
    console.log('[worker] fetching', path);

    fetch(path)
      .then((r) => r.text())
      .then((c) => {
        console.log('[worker]', 'got content at ' + path);
        self.files[path] = c;

        postMessage(JSON.stringify({
          event: 'try-again'
        }));
      })
      .catch((e) => {
        console.error('[worker]', 'fetching', path, e);
        self.files[path] = { error: e };
      });

    return { error: '__parity_tryAgain' };
  }

  console.log(`[worker] path ${path} not found...`);
  return { error: 'File not found' };
}

function compile (data) {
  const { sourcecode, build } = data;
  const { longVersion } = build;

  if (self.lastCompile.sourcecode === sourcecode && self.lastCompile.longVersion === longVersion) {
    return postMessage(JSON.stringify({
      event: 'compiled',
      data: self.lastCompile.result
    }));
  }

  fetchSolc(build)
    .then((compiler) => {
      const input = {
        '': sourcecode
      };

      const compiled = compiler.compile({ sources: input }, 0, findImports);

      self.lastCompile = {
        version: longVersion, result: compiled,
        sourcecode
      };

      postMessage(JSON.stringify({
        event: 'compiled',
        data: compiled
      }));
    });
}

function load (build) {
  postMessage(JSON.stringify({
    event: 'loading',
    data: true
  }));

  fetchSolc(build)
    .then(() => {
      postMessage(JSON.stringify({
        event: 'loading',
        data: false
      }));
    })
    .catch(() => {
      postMessage(JSON.stringify({
        event: 'loading',
        data: false
      }));
    });
}

function fetchSolc (build) {
  const { path, longVersion } = build;

  if (self.solcVersions[path]) {
    return Promise.resolve(self.solcVersions[path]);
  }

  const URL = `https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/${path}`;
  console.log(`[worker] fetching solc-bin ${longVersion} at ${URL}`);

  return fetch(URL)
    .then((r) => r.text())
    .then((code) => {
      const solcCode = code.replace(/^var Module;/, 'var Module=self.__solcModule;');
      self.__solcModule = {};

      console.log(`[worker] evaluating ${longVersion}`);

      // eslint-disable-next-line no-eval
      eval(solcCode);

      console.log(`[worker] done evaluating ${longVersion}`);

      const compiler = solc(self.__solcModule);
      self.solcVersions[path] = compiler;
      return compiler;
    })
    .catch((e) => {
      console.error('fetching solc', e);
    });
}
