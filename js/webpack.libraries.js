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

// Run with `webpack --config webpack.libraries.js --progress`

const HappyPack = require('happypack');
const path = require('path');
const webpack = require('webpack');

const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';
const DEST = process.env.BUILD_DEST || '.build';

module.exports = {
  context: path.join(__dirname, './src'),
  entry: {
    // library
    'inject': ['./web3.js'],
    'web3': ['./web3.js'],
    'parity': ['./parity.js']
  },
  output: {
    path: path.join(__dirname, DEST),
    filename: '[name].js'
  },
  module: {
    loaders: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        loader: 'happypack/loader?id=js'
      },
      {
        test: /\.json$/,
        loaders: ['json']
      },
      {
        test: /\.html$/,
        loader: 'file?name=[name].[ext]'
      }
    ]
  },
  plugins: (function () {
    const plugins = [
      new HappyPack({
        id: 'js',
        threads: 4,
        loaders: [ 'babel' ]
      }),
      new webpack.DefinePlugin({
        'process.env': {
          NODE_ENV: JSON.stringify(ENV),
          RPC_ADDRESS: JSON.stringify(process.env.RPC_ADDRESS),
          PARITY_URL: JSON.stringify(process.env.PARITY_URL),
          LOGGING: JSON.stringify(!isProd)
        }
      })
    ];

    if (isProd) {
      plugins.push(new webpack.optimize.OccurrenceOrderPlugin(false));
      plugins.push(new webpack.optimize.DedupePlugin());
      plugins.push(new webpack.optimize.UglifyJsPlugin({
        screwIe8: true,
        compress: {
          warnings: false
        },
        output: {
          comments: false
        }
      }));
    }

    return plugins;
  }())
};
