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

const HappyPack = require('happypack');
const webpack = require('webpack');

const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';
const DEST = process.env.BUILD_DEST || '.build';

let modules = [
  'babel-polyfill',
  'bignumber.js',
  'blockies',
  'brace',
  'browserify-aes',
  'chart.js',
  'ethereumjs-tx',
  'lodash',
  'material-ui',
  'mobx',
  'mobx-react',
  'moment',
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

if (!isProd) {
  modules = modules.concat([
    'webpack-dev-server/client?http://localhost:3000',
    'react-hot-loader', 'core-js', 'core-js/library'
  ]);
}

module.exports = {
  entry: {
    vendor: modules
  },
  module: {
    loaders: [
      {
        test: /\.json$/,
        loaders: ['json']
      },
      {
        test: /\.js$/,
        include: /(ethereumjs-tx)/,
        loaders: [ 'happypack/loader?id=js' ]
      }
    ]
  },
  output: {
    filename: '[name].js',
    path: `${DEST}/`,
    library: '[name]_lib'
  },
  plugins: (function () {
    const plugins = [
      new webpack.DllPlugin({
        name: '[name]_lib',
        path: `${DEST}/[name]-manifest.json`
      }),

      new webpack.DefinePlugin({
        'process.env': {
          NODE_ENV: JSON.stringify(ENV)
        }
      }),

      new HappyPack({
        id: 'js',
        threads: 4,
        loaders: ['babel']
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
