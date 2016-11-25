
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

const webpack = require('webpack');
const path = require('path');
const WebpackErrorNotificationPlugin = require('webpack-error-notification');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const Shared = require('./shared');
const DAPPS = require('../src/dapps');

const FAVICON = path.resolve(__dirname, '../assets/images/parity-logo-black-no-text.png');

const DEST = process.env.BUILD_DEST || '.build';
const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';

module.exports = {
  debug: !isProd,
  cache: !isProd,
  devtool: isProd ? '#eval' : '#cheap-module-eval-source-map',

  context: path.join(__dirname, '../src'),
  entry: Object.assign({}, Shared.dappsEntry, {
    index: './index.js'
  }),
  output: {
    publicPath: '/',
    path: path.join(__dirname, '../', DEST),
    filename: '[name].[hash].js'
  },

  module: {
    loaders: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        loaders: [ 'happypack/loader?id=js' ]
      },
      {
        test: /\.js$/,
        include: /node_modules\/material-ui-chip-input/,
        loader: 'babel'
      },
      {
        test: /\.json$/,
        loaders: ['json']
      },
      {
        test: /\.html$/,
        loader: 'file?name=[name].[ext]!extract-loader!html-loader'
      },

      {
        test: /\.css$/,
        include: [/src/],
        loaders: [ 'happypack/loader?id=css' ]
      },
      {
        test: /\.css$/,
        exclude: [/src/],
        loader: 'style!css'
      },
      {
        test: /\.(png|jpg)$/,
        loader: 'file-loader?name=[name].[hash].[ext]'
      },
      {
        test: /\.(woff(2)|ttf|eot|svg|otf)(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        loader: 'file-loader'
      }
    ],
    noParse: [
      /node_modules\/sinon/
    ]
  },

  resolve: {
    root: path.join(__dirname, '../node_modules'),
    fallback: path.join(__dirname, '../node_modules'),
    extensions: ['', '.js', '.jsx'],
    unsafeCache: true
  },
  resolveLoaders: {
    root: path.join(__dirname, '../node_modules'),
    fallback: path.join(__dirname, '../node_modules')
  },

  htmlLoader: {
    root: path.resolve(__dirname, '../assets/images'),
    attrs: ['img:src', 'link:href']
  },

  postcss: Shared.postcss,

  plugins: (function () {
    const plugins = Shared.getPlugins().concat([
      new CopyWebpackPlugin([{ from: './error_pages.css', to: 'styles.css' }], {}),
      new WebpackErrorNotificationPlugin(),

      new webpack.DllReferencePlugin({
        context: '.',
        manifest: require(`../${DEST}/vendor-manifest.json`)
      }),

      new HtmlWebpackPlugin({
        title: 'Parity',
        filename: 'index.html',
        template: './index.ejs',
        favicon: FAVICON,
        chunks: [ isProd ? null : 'commons', 'index' ]
      })
    ], DAPPS.map((dapp) => {
      return new HtmlWebpackPlugin({
        title: dapp.title,
        filename: dapp.name + '.html',
        template: './dapps/index.ejs',
        favicon: FAVICON,
        secure: dapp.secure,
        chunks: [ isProd ? null : 'commons', dapp.name ]
      });
    }));

    if (!isProd) {
      plugins.push(
        new webpack.optimize.CommonsChunkPlugin({
          filename: 'commons.[hash].js',
          name: 'commons'
        })
      );
    }

    return plugins;
  }())
};
