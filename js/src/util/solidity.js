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

import solc from 'solc/browser-wrapper';

export default class SolidityUtils {
  static compile (data, compiler) {
    const { sourcecode, build, optimize, files, name = '' } = data;

    const start = Date.now();

    console.log('[solidity] compiling...');

    const input = {
      [ name ]: sourcecode
    };

    const findFiles = (path) => {
      const file = files.find((f) => f.name === path);

      if (file) {
        return { contents: file.sourcecode };
      } else {
        return { error: 'File not found' };
      }
    };

    const compiled = compiler.compile({ sources: input }, optimize ? 1 : 0, findFiles);

    const time = Math.round((Date.now() - start) / 100) / 10;

    console.log(`[solidity] done compiling in ${time}s`);

    compiled.version = build.longVersion;
    compiled.sourcecode = sourcecode;

    return compiled;
  }

  static getCompiler (build, _fetcher) {
    const { longVersion, path } = build;

    const URL = `https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/${path}`;

    const fetcher = typeof _fetcher === 'function'
      ? _fetcher
      : (url) => fetch(url);

    const isWorker = typeof window !== 'object';

    return fetcher(URL)
      .then((r) => r.text())
      .then((code) => {
        // `window` for main thread, `self` for workers
        const _self = isWorker ? self : window;

        _self.Module = {};

        const solcCode = code.replace('var Module;', `var Module=${isWorker ? 'self' : 'window'}.Module;`);

        console.log(`[solidity] evaluating ${longVersion}`);

        try {
          // eslint-disable-next-line no-eval
          eval(solcCode);
        } catch (e) {
          return Promise.reject(e);
        }

        console.log(`[solidity] done evaluating ${longVersion}`);

        const compiler = solc(_self.Module);

        delete _self.Module;

        return compiler;
      });
  }
}
