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
const fs = require('fs');
const HappyPack = require('happypack');

const postcssImport = require('postcss-import');
const postcssNested = require('postcss-nested');
const postcssVars = require('postcss-simple-vars');
const rucksack = require('rucksack-css');
const CircularDependencyPlugin = require('circular-dependency-plugin');
const ProgressBarPlugin = require('progress-bar-webpack-plugin');

const EMBED = process.env.EMBED;
const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';
const isAnalize = process.env.WPANALIZE === '1';

function getBabelrc () {
  const babelrc = JSON.parse(fs.readFileSync(path.resolve(__dirname, '../.babelrc')));

  const es2015Index = babelrc.presets.findIndex((p) => p === 'es2015');

  // [ "es2015", { "modules": false } ]
  babelrc.presets[es2015Index] = [ 'es2015', { modules: false } ];
  babelrc['babelrc'] = false;

  const BABEL_PRESET_ENV = process.env.BABEL_PRESET_ENV;
  const npmStart = process.env.npm_lifecycle_event === 'start';
  const npmStartApp = process.env.npm_lifecycle_event === 'start:app';

  if (BABEL_PRESET_ENV && (npmStart || npmStartApp)) {
    console.log('using babel-preset-env');

    babelrc.presets = [
      // 'es2017',
      'stage-0', 'react',
      [
        'env',
        {
          targets: { browsers: ['last 2 Chrome versions'] },
          modules: false,
          loose: true,
          useBuiltIns: true
        }
      ]
    ];
  }

  return babelrc;
}

function getPlugins (_isProd = isProd) {
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

  const plugins = (isAnalize
    ? []
    : [
      new ProgressBarPlugin({
        format: '[:msg] [:bar] ' + ':percent' + ' (:elapsed seconds)'
      })
    ]).concat([
      new HappyPack({
        id: 'css',
        threads: 4,
        loaders: [
          'style-loader',
          'css-loader?modules&sourceMap&importLoaders=1&localIdentName=[name]__[local]___[hash:base64:5]',
          'postcss-loader'
        ],
        verbose: !isAnalize
      }),

      new HappyPack({
        id: 'babel-js',
        threads: 4,
        loaders: [ isProd ? 'babel-loader' : 'babel-loader?cacheDirectory=true' ],
        verbose: !isAnalize
      }),

      new webpack.DefinePlugin({
        'process.env': {
          EMBED: JSON.stringify(EMBED),
          NODE_ENV: JSON.stringify(ENV),
          RPC_ADDRESS: JSON.stringify(process.env.RPC_ADDRESS),
          PARITY_URL: JSON.stringify(process.env.PARITY_URL),
          DAPPS_URL: JSON.stringify(process.env.DAPPS_URL),
          LOGGING: JSON.stringify(!isProd)
        }
      }),

      new webpack.LoaderOptionsPlugin({
        minimize: isProd,
        debug: !isProd,
        options: {
          context: path.join(__dirname, '../src'),
          postcss: postcss,
          babel: getBabelrc()
        }
      }),

      new webpack.optimize.OccurrenceOrderPlugin(!_isProd),

      new CircularDependencyPlugin({
        exclude: /node_modules/,
        failOnError: true
      })
    ]);

  if (_isProd) {
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
  return [];
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
  getBabelrc: getBabelrc,
  getPlugins: getPlugins,
  dappsEntry: getDappsEntry(),
  addProxies: addProxies
};
