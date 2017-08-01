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

// Run with `webpack --config webpack.libraries.js`

const path = require('path');

const rulesEs6 = require('./rules/es6');
const rulesParity = require('./rules/parity');
const Shared = require('./shared');

const DEST = process.env.BUILD_DEST || '.build';

module.exports = ['inject', 'parity', 'web3'].map((entryName) => {
  return {
    context: path.join(__dirname, '../src'),
    entry: {
      [entryName]: ['./inject.js']
    },
    output: {
      path: path.join(__dirname, '../', DEST),
      filename: '[name].js',
      library: '[name].js',
      libraryTarget: 'umd'
    },

    resolve: {
      alias: {
        '~': path.resolve(__dirname, '..'),
        '@parity/abi': path.resolve(__dirname, '../node_modules/@parity/abi'),
        '@parity/api': path.resolve(__dirname, '../node_modules/@parity/api'),
        '@parity/etherscan': path.resolve(__dirname, '../node_modules/@parity/etherscan'),
        '@parity/jsonrpc': path.resolve(__dirname, '../node_modules/@parity/jsonrpc'),
        '@parity/shared': path.resolve(__dirname, '../node_modules/@parity/shared'),
        '@parity/ui': path.resolve(__dirname, '../node_modules/@parity/ui'),
        '@parity/wordlist': path.resolve(__dirname, '../node_modules/@parity/wordlist'),
        '@parity': path.resolve(__dirname, '../packages')
      }
    },

    node: {
      fs: 'empty'
    },

    module: {
      rules: [
        rulesParity,
        rulesEs6,
        {
          test: /\.js$/,
          exclude: /node_modules/,
          use: [ 'babel-loader' ]
        },
        {
          test: /\.json$/,
          use: [ 'json-loader' ]
        },
        {
          test: /\.html$/,
          use: [ {
            loader: 'file-loader',
            options: {
              name: '[name].[ext]'
            }
          } ]
        }
      ]
    },
    plugins: Shared.getPlugins()
  };
});
