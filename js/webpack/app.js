
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

const fs = require('fs');
const path = require('path');
const rimraf = require('rimraf');
const flatten = require('lodash.flatten');
// const ReactIntlAggregatePlugin = require('react-intl-aggregate-webpack-plugin');
const ExtractTextPlugin = require('extract-text-webpack-plugin');
const WebpackErrorNotificationPlugin = require('webpack-error-notification');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');

const rulesEs6 = require('./rules/es6');
const rulesParity = require('./rules/parity');
const Shared = require('./shared');

const DAPPS_BUILTIN = require('@parity/shared/lib/config/dappsBuiltin.json');
const DAPPS_VIEWS = require('@parity/shared/lib/config/dappsViews.json');
const DAPPS_ALL = []
  .concat(DAPPS_BUILTIN, DAPPS_VIEWS)
  .filter((dapp) => !dapp.skipBuild)
  .filter((dapp) => dapp.package);

const FAVICON = path.resolve(__dirname, '../node_modules/@parity/shared/assets/images/parity-logo-black-no-text.png');

const DEST = process.env.BUILD_DEST || '.build';
const ENV = process.env.NODE_ENV || 'development';
const EMBED = process.env.EMBED;

const isProd = ENV === 'production';
const isEmbed = EMBED === '1' || EMBED === 'true';

const entry = isEmbed
  ? { embed: ['babel-polyfill', './embed.js'] }
  : { bundle: ['babel-polyfill', './index.parity.js'] };

module.exports = {
  cache: !isProd,
  devtool: isProd
    ? false
    : '#eval',
  context: path.join(__dirname, '../src'),
  entry,
  output: {
    path: path.join(__dirname, '../', DEST),
    filename: '[name].js'
  },

  module: {
    rules: [
      rulesParity,
      rulesEs6,
      {
        test: /\.js$/,
        exclude: /node_modules/,
        use: [{
          loader: 'happypack/loader',
          options: {
            id: 'babel'
          }
        }]
      },
      {
        test: /\.json$/,
        use: ['json-loader']
      },
      {
        test: /\.ejs$/,
        use: ['ejs-loader']
      },
      {
        test: /\.md$/,
        use: ['html-loader', 'markdown-loader']
      },
      {
        test: /\.css$/,
        include: /semantic-ui-css|@parity\/ui/,
        use: ExtractTextPlugin.extract({
          fallback: 'style-loader',
          use: [
            {
              loader: 'css-loader',
              options: {
                minimize: isProd
              }
            }
          ]
        })
      },
      {
        test: /\.css$/,
        exclude: /semantic-ui-css|@parity\/ui/,
        use: ExtractTextPlugin.extract({
          fallback: 'style-loader',
          use: [
            {
              loader: 'css-loader',
              options: {
                importLoaders: 1,
                localIdentName: '[name]_[local]_[hash:base64:10]',
                minimize: isProd,
                modules: true
              }
            },
            {
              loader: 'postcss-loader',
              options: {
                sourceMap: isProd,
                plugins: [
                  require('postcss-import'),
                  require('postcss-nested'),
                  require('postcss-simple-vars')
                ]
              }
            }
          ]
        })
      },
      {
        test: /\.(png|jpg|svg|woff|woff2|ttf|eot|otf)(\?.*)?$/,
        use: {
          loader: 'file-loader',
          options: {
            name: '[name].[hash:10].[ext]',
            outputPath: '',
            publicPath: '',
            useRelativePath: false
          }
        }
      }
    ],
    noParse: [
      /node_modules\/sinon/
    ]
  },

  resolve: {
    modules: [
      path.join(__dirname, '../node_modules')
    ],
    extensions: ['.json', '.js', '.jsx'],
    unsafeCache: true
  },

  node: {
    fs: 'empty'
  },

  plugins: (function () {
    let plugins = Shared.getPlugins().concat(
      new WebpackErrorNotificationPlugin(),
      new ExtractTextPlugin({
        filename: `${isEmbed ? 'embed' : 'bundle'}.css`
      }),
    );

    if (!isEmbed) {
      plugins = [].concat(
        plugins,

        new HtmlWebpackPlugin({
          title: 'Parity',
          filename: 'index.html',
          template: './index.parity.ejs',
          favicon: FAVICON,
          chunks: ['bundle']
        }),

        new CopyWebpackPlugin(
          flatten([
            {
              from: path.join(__dirname, '../src/dev.web3.html'),
              to: 'dev.web3/index.html'
            },
            {
              from: path.join(__dirname, '../src/dev.parity.html'),
              to: 'dev.parity/index.html'
            },
            {
              from: path.join(__dirname, '../src/error_pages.css'),
              to: 'styles.css'
            },
            {
              from: path.join(__dirname, '../src/index.electron.js'),
              to: 'electron.js'
            },
            {
              from: path.join(__dirname, '../package.electron.json'),
              to: 'package.json'
            },
            flatten(
              DAPPS_ALL
                .map((dapp) => {
                  const dir = path.join(__dirname, '../node_modules', dapp.package);

                  if (!fs.existsSync(dir)) {
                    return null;
                  }

                  if (!fs.existsSync(path.join(dir, 'dist'))) {
                    rimraf.sync(path.join(dir, 'node_modules'));

                    return {
                      from: path.join(dir),
                      to: `dapps/${dapp.id}/`
                    };
                  }

                  return [
                    'icon.png', 'index.html', 'dist.css', 'dist.js',
                    isProd ? null : 'dist.css.map',
                    isProd ? null : 'dist.js.map'
                  ]
                    .filter((file) => file)
                    .map((file) => path.join(dir, file))
                    .filter((from) => fs.existsSync(from))
                    .map((from) => ({
                      from,
                      to: `dapps/${dapp.id}/`
                    }))
                    .concat({
                      from: path.join(dir, 'dist'),
                      to: `dapps/${dapp.id}/dist/`
                    });
                })
                .filter((copy) => copy)
            )
          ]),
          {}
        )
      );
    }

    if (isEmbed) {
      plugins.push(
        new HtmlWebpackPlugin({
          title: 'Parity Bar',
          filename: 'embed.html',
          template: './index.parity.ejs',
          favicon: FAVICON,
          chunks: ['embed']
        })
      );
    }

    return plugins;
  }())
};
