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

const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const packageJson = require('../package.json');

const rulesEs6 = require('./rules/es6');
const rulesParity = require('./rules/parity');
const Shared = require('./shared');

const LIBRARY = process.env.LIBRARY;

if (!LIBRARY) {
  console.error('$LIBRARY environment variable not defined');
  process.exit(-1);
}

const SRC = LIBRARY.toLowerCase();
const OUTPUT_PATH = path.join(__dirname, '../.npmjs', SRC);

const TEST_CONTEXT = SRC === 'parity'
  ? '../npm/parity/test/'
  : `../packages/${SRC}/`;

console.log(`Building ${LIBRARY} from library.${SRC}.js to .npmjs/${SRC}`);

module.exports = {
  context: path.join(__dirname, '../src'),
  target: 'node',
  entry: `library.${SRC}.js`,
  output: {
    path: OUTPUT_PATH,
    filename: 'library.js',
    library: LIBRARY,
    libraryTarget: 'umd',
    umdNamedDefine: true
  },
  externals: {
    'node-fetch': 'node-fetch'
  },
  module: {
    rules: [
      rulesParity,
      rulesEs6,
      {
        test: /(\.jsx|\.js)$/,
        use: ['babel-loader'],
        exclude: /node_modules/
      }
    ]
  },
  node: {
    fs: 'empty'
  },
  resolve: {
    alias: {},
    modules: [
      path.join(__dirname, '../node_modules')
    ],
    extensions: ['.json', '.js', '.jsx']
  },

  plugins: Shared.getPlugins().concat([
    new CopyWebpackPlugin([
      {
        from: `../npm/${SRC}/package.json`,
        to: 'package.json',
        transform: function (content, path) {
          const json = JSON.parse(content.toString());

          json.devDependencies.chai = packageJson.devDependencies.chai;
          json.devDependencies.mocha = packageJson.devDependencies.mocha;
          json.devDependencies.nock = packageJson.devDependencies.nock;
          json.scripts.test = 'mocha \'test/*.spec.js\'';
          json.version = packageJson.version;

          return new Buffer(JSON.stringify(json, null, '  '), 'utf-8');
        }
      },
      {
        from: '../LICENSE'
      },

      // Copy the base test config
      {
        from: '../npm/test',
        to: 'test'
      },

      // Copy the actual tests
      {
        context: TEST_CONTEXT,
        from: '**/*.spec.js',
        to: 'test',
        transform: function (content, path) {
          let output = content.toString();

          // Don't skip tests
          output = output.replace(/describe\.skip/, 'describe');

          // Require parent library
          output = output.replace('require(\'./\')', 'require(\'../\')');

          return new Buffer(output, 'utf-8');
        }
      },
      {
        from: `../npm/${SRC}/README.md`,
        to: 'README.md'
      }
    ], { copyUnmodified: true })
  ])
};
