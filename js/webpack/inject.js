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

const rulesEs6 = require('./rules/es6');
const rulesParity = require('./rules/parity');
const Shared = require('./shared');

const isProd = process.env.NODE_ENV === 'production';
const DEST = process.env.BUILD_DEST || '.build';

module.exports = {
  context: path.join(__dirname, '../src'),
  devtool: isProd
    ? false
    : '#eval',
  entry: {
    inject: ['./inject.js'],
    parity: ['./inject.script.js'],
    web3: ['./inject.script.js']
  },
  output: {
    path: path.join(__dirname, '../', DEST),
    filename: '[name].js',
    library: '[name].js',
    libraryTarget: 'umd'
  },

  resolve: {
    alias: {}
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
        use: [ {
          loader: 'happypack/loader',
          options: {
            id: 'babel'
          }
        } ]
      },
      {
        test: /\.json$/,
        use: ['json-loader']
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
