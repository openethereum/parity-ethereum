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

const path = require('path');
const webpack = require('webpack');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const packageJson = require('./package.json');

const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';

module.exports = {
  context: path.join(__dirname, './src'),
  entry: 'library.js',
  output: {
    path: path.join(__dirname, '.npmjs'),
    filename: 'library.js',
    libraryTarget: 'commonjs'
  },
  module: {
    loaders: [
      {
        test: /(\.jsx|\.js)$/,
        loader: 'babel',
        exclude: /node_modules/
      }
    ]
  },
  resolve: {
    root: path.resolve('./src'),
    extensions: ['', '.js']
  },
  plugins: (function () {
    const plugins = [
      new CopyWebpackPlugin([
        {
          from: '../parity.package.json',
          to: 'package.json',
          transform: function (content, path) {
            const json = JSON.parse(content.toString());
            json.version = packageJson.version;
            return new Buffer(JSON.stringify(json, null, '  '), 'utf-8');
          }
        },
        {
          from: '../LICENSE'
        },
        {
          from: '../parity.md',
          to: 'README.md'
        }
      ], { copyUnmodified: true })
    ];

    if (isProd) {
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
