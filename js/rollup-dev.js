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

import babel from 'rollup-plugin-babel';
import cjs from 'rollup-plugin-commonjs';
import json from 'rollup-plugin-json';
import multiEntry from 'rollup-plugin-multi-entry';
import globals from 'rollup-plugin-node-globals';
import resolve from 'rollup-plugin-node-resolve';
import postcss from 'rollup-plugin-postcss';
import replace from 'rollup-plugin-replace';
import url from 'rollup-plugin-url';

export default {
  dest: 'dist/app.js',
  entry: [
    // dapps
    'src/dapps/gavcoin.js',
    'src/dapps/registry.js',
    'src/dapps/tokenreg.js',

    // libraries
    'src/parity.js',

    // app(s)
    'src/app.js'
  ],
  format: 'cjs',
  plugins: [
    multiEntry(),
    babel({
      babelrc: false,
      exclude: 'node_modules/**',
      presets: [ 'es2017', 'es2016', 'es2015-rollup', 'stage-0', 'react' ],
      runtimeHelpers: true
    }),
    cjs({
      exclude: 'node_modules/process-es6/**',
      include: [
        'node_modules/fbjs/**',
        'node_modules/react/**',
        'node_modules/react-dom/**'
      ]
    }),
    replace({
      'process.env.NODE_ENV': JSON.stringify('development')
    }),
    json(),
    postcss(),
    globals(),
    url({
      limit: 0,
      publicPath: 'dist/'
    }),
    resolve({
      browser: true,
      main: true
    })
  ],
  sourceMap: true
};
