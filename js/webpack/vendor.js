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

const webpack = require('webpack');
const path = require('path');

const Shared = require('./shared');

const ENV = process.env.NODE_ENV || 'development';
const DEST = process.env.BUILD_DEST || '.build';

let modules = [
  'babel-polyfill',
  'bignumber.js',
  'blockies',
  'brace',
  'browserify-aes',
  'ethereumjs-tx',
  'lodash',
  'material-ui',
  'material-ui-chip-input',
  'mobx',
  'mobx-react',
  'moment',
  'phoneformat.js',
  'react',
  'react-dom',
  'react-redux',
  'react-router',
  'react-router-redux',
  'recharts',
  'redux',
  'redux-thunk',
  'scryptsy'
];

module.exports = {
  entry: {
    vendor: modules
  },
  module: {
    rules: [
      {
        test: /\.json$/,
        use: [ 'json-loader' ]
      },
      {
        test: /\.js$/,
        include: /(ethereumjs-tx)/,
        use: [ 'babel-loader' ]
      }
    ]
  },

  resolve: {
    alias: {
      '~': path.resolve(__dirname, '../src')
    }
  },

  output: {
    filename: '[name].js',
    path: path.resolve(__dirname, '../', `${DEST}/`),
    library: '[name]_lib'
  },
  plugins: Shared.getPlugins().concat([
    new webpack.DllPlugin({
      name: '[name]_lib',
      path: path.resolve(__dirname, '../', `${DEST}/[name]-manifest.json`)
    }),

    new webpack.DefinePlugin({
      'process.env': {
        NODE_ENV: JSON.stringify(ENV)
      }
    })
  ])
};
