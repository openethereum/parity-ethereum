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

const HappyPack = require('happypack');
const PackageJson = require('../package.json');

const EMBED = process.env.EMBED;
const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';
const UI_VERSION = PackageJson
  .version
  .split('.')
  .map((part, index) => {
    if (index !== 2) {
      return part;
    }

    return `${parseInt(part, 10) + 1}`;
  })
  .join('.');

function getPlugins (_isProd = isProd) {
  const plugins = [
    new webpack.DefinePlugin({
      'process.env': {
        EMBED: JSON.stringify(EMBED),
        NODE_ENV: JSON.stringify(ENV),
        RPC_ADDRESS: JSON.stringify(process.env.RPC_ADDRESS),
        PARITY_URL: JSON.stringify(process.env.PARITY_URL),
        DAPPS_URL: JSON.stringify(process.env.DAPPS_URL),
        LOGGING: JSON.stringify(!isProd),
        UI_VERSION: JSON.stringify(UI_VERSION)
      }
    }),
    new HappyPack({
      id: 'babel',
      threads: 4,
      loaders: ['babel-loader']
    })
  ];

  if (_isProd) {
    plugins.push(
      new webpack.optimize.ModuleConcatenationPlugin(),
      new webpack.optimize.UglifyJsPlugin({
        sourceMap: true,
        screwIe8: true,
        compress: {
          warnings: false
        },
        output: {
          comments: false
        }
      })
    );
  }

  return plugins;
}

function addProxies (app) {
  const proxy = require('http-proxy-middleware');

  app.use('/api', proxy({
    target: 'http://127.0.0.1:8180',
    changeOrigin: true,
    autoRewrite: true
  }));

  app.use('/app', proxy({
    target: 'http://127.0.0.1:8545',
    changeOrigin: true,
    pathRewrite: {
      '^/app': ''
    }
  }));

  app.use('/parity-utils', proxy({
    target: 'http://127.0.0.1:3000',
    changeOrigin: true,
    pathRewrite: {
      '^/parity-utils': ''
    }
  }));

  app.use('/rpc', proxy({
    target: 'http://127.0.0.1:8545',
    changeOrigin: true
  }));
}

module.exports = {
  getPlugins,
  addProxies
};
