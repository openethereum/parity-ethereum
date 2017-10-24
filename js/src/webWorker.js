// Copyright 2015-2017 Parity Technologies (UK) Ltd.
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

import registerPromiseWorker from 'promise-worker/register';
import { Signer } from '~/util/signer';
import SolidityUtils from '~/util/solidity';

registerPromiseWorker((msg) => {
  return handleMessage(msg);
});

self.compiler = {
  version: null,
  compiler: null
};
self.files = {};

function handleMessage (message) {
  switch (message.action) {
    case 'compile':
      return compile(message.data);

    case 'load':
      return getCompiler(message.data).then(() => 'ok');

    case 'setFiles':
      return setFiles(message.data);

    case 'getSignerSeed':
      return getSignerSeed(message.data);

    default:
      console.warn(`unknown action "${message.action}"`);
      return null;
  }
}

function getSignerSeed (data) {
  console.log('deriving seed from service-worker');
  const { wallet, password } = data;

  return Signer.getSeed(wallet, password);
}

function compile (data) {
  const { build } = data;

  return getCompiler(build)
    .then((compiler) => {
      console.warn('compiling');
      return SolidityUtils.compile(data, compiler);
    })
    .then((result) => {
      console.warn('result in worker', result);
      return result;
    })
    .catch((error) => {
      console.error('error in worker', error);
      throw error;
    });
}

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

  return 'ok';
}

function getCompiler (build) {
  const { longVersion } = build;

  if (self.compiler.version !== longVersion) {
    self.compiler.version = longVersion;
    self.compiler.compiler = SolidityUtils
      .getCompiler(build)
      .then((compiler) => {
        if (self.compiler.version === longVersion) {
          self.compiler.compiler = compiler;
        }

        return compiler;
      });
  }

  return Promise.resolve(self.compiler.compiler);
}
