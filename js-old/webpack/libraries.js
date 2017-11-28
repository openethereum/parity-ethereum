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

const Shared = require('./shared');

const DEST = process.env.BUILD_DEST || '.build';
const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';

module.exports = {
  context: path.join(__dirname, '../src'),
  entry: {
    // library
    'inject': ['./inject.js'],
    'web3': ['./web3.js'],
    'parity': ['./parity.js']
  },
  output: {
    path: path.join(__dirname, '../', DEST),
    filename: '[name].js',
    library: '[name].js',
    libraryTarget: 'umd'
  },

  resolve: {
    alias: {
      '~': path.resolve(__dirname, '../src')
    }
  },

  node: {
    fs: 'empty'
  },

  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        // use: [ 'happypack/loader?id=js' ]
        use: isProd ? ['babel-loader'] : [
          // 'react-hot-loader',
          'babel-loader?cacheDirectory=true'
        ]
      },
      {
        test: /\.js$/,
        include: /(ethereumjs-tx|wordlist)/,
        use: 'babel-loader'
      },
      {
        test: /\.json$/,
        use: [ 'json-loader' ]
      },
      {
        test: /\.html$/,
        use: [ 'file-loader?name=[name].[ext]' ]
      }
    ]
  },
  plugins: Shared.getPlugins()
};
