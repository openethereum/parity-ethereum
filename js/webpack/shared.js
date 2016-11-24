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
const HappyPack = require('happypack');

const postcssImport = require('postcss-import');
const postcssNested = require('postcss-nested');
const postcssVars = require('postcss-simple-vars');
const rucksack = require('rucksack-css');

const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';

function getPlugins (_isProd = isProd) {
  const plugins = [
    new HappyPack({
      id: 'css',
      threads: 4,
      loaders: [
        'style',
        'css?modules&sourceMap&importLoaders=1&localIdentName=[name]__[local]___[hash:base64:5]',
        'postcss'
      ]
    }),

    new HappyPack({
      id: 'js',
      threads: 4,
      loaders: _isProd ? ['babel'] : [
        'react-hot',
        'babel?cacheDirectory=true'
      ]
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

  if (_isProd) {
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
}

function getDappsEntry () {
  const DAPPS = require('../src/dapps');

  return DAPPS.reduce((_entry, dapp) => {
    _entry[dapp.name] = './dapps/' + dapp.entry;
    return _entry;
  }, {});
}

const postcss = [
  postcssImport({
    addDependencyTo: webpack
  }),
  postcssNested({}),
  postcssVars({
    unknown: function (node, name, result) {
      node.warn(result, `Unknown variable ${name}`);
    }
  }),
  rucksack({
    autoprefixer: true
  })
];

const proxies = [
  {
    context: (pathname, req) => {
      return pathname === '/' && req.method === 'HEAD';
    },
    target: 'http://127.0.0.1:8180',
    changeOrigin: true,
    autoRewrite: true
  },
  {
    context: '/api',
    target: 'http://127.0.0.1:8080',
    changeOrigin: true,
    autoRewrite: true
  },
  {
    context: '/app',
    target: 'http://127.0.0.1:8080',
    changeOrigin: true,
    pathRewrite: {
      '^/app': ''
    }
  },
  {
    context: '/parity-utils',
    target: 'http://127.0.0.1:3000',
    changeOrigin: true,
    pathRewrite: {
      '^/parity-utils': ''
    }
  },
  {
    context: '/rpc',
    target: 'http://127.0.0.1:8080',
    changeOrigin: true
  }
];

module.exports = {
  getPlugins: getPlugins,
  dappsEntry: getDappsEntry(),
  postcss: postcss,
  proxies: proxies
};
