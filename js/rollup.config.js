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
import builtins from 'rollup-plugin-node-builtins';
import cjs from 'rollup-plugin-commonjs';
import json from 'rollup-plugin-json';
import globals from 'rollup-plugin-node-globals';
import resolve from 'rollup-plugin-node-resolve';
import postcss from 'rollup-plugin-postcss';
import replace from 'rollup-plugin-replace';
import uglify from 'rollup-plugin-uglify';
import url from 'rollup-plugin-url';

const { NODE_ENV, dapp, src } = process.env;

const target = dapp ? `dapps/${dapp}/${dapp}` : src;
const isProd = process.env.NODE_ENV === 'production';

const dest = `dist/${target}.js`;
const entry = `src/${target}.js`;

console.log(`Building ${entry} to ${dest}`);

const config = {
  dest,
  entry,
  format: 'cjs',
  sourceMap: true,
  plugins: [
    postcss({}),
    json(),
    url({
      limit: 1,
      publicPath: 'dist/'
    }),
    babel({
      babelrc: false,
      exclude: 'node_modules/**',
      presets: [ 'es2017', 'es2016', 'es2015-rollup', 'stage-0', 'react' ],
      runtimeHelpers: true
    }),
    builtins(),
    resolve({
      browser: true,
      jsnext: true,
      skip: ['crypto'],
      preferBuiltins: false
    }),
    cjs({
      include: 'node_modules/**',
      exclude: [
        'node_modules/buffer-es6/**',
        'node_modules/process-es6/**',
        'node_modules/moment/**',
        'node_modules/redux/node_modules/symbol-observable/**',
        'node_modules/rollup-plugin-node-builtins/**',
        'node_modules/rollup-plugin-node-globals/**'
      ],
      namedExports: {
        'es6-promise': ['polyfill'],
        'js-sha3': ['keccak_256'],
        'lodash': ['isEqual'],
        'react': ['Component', 'createElement', 'PropTypes'],
        'utf8': ['encode', 'decode']
      }
    }),
    // replace({
    //   'process.env.NODE_ENV': JSON.stringify(NODE_ENV || 'development')
    // }),
    globals()
  ]
};

if (isProd) {
  config.plugins.push(uglify());
}

export default config;
