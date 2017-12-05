
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
const ReactIntlAggregatePlugin = require('react-intl-aggregate-webpack-plugin');
const WebpackErrorNotificationPlugin = require('webpack-error-notification');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const ExtractTextPlugin = require('extract-text-webpack-plugin');
const ServiceWorkerWebpackPlugin = require('serviceworker-webpack-plugin');
const ScriptExtHtmlWebpackPlugin = require('script-ext-html-webpack-plugin');

const Shared = require('./shared');
const DAPPS = require('../src/views/Dapps/builtin.json');

const FAVICON = path.resolve(__dirname, '../assets/images/parity-logo-black-no-text.png');

const DEST = process.env.BUILD_DEST || '.build';
const ENV = process.env.NODE_ENV || 'development';
const EMBED = process.env.EMBED;

const isProd = ENV === 'production';
const isEmbed = EMBED === '1' || EMBED === 'true';
const isAnalize = process.env.WPANALIZE === '1';

const entry = isEmbed
  ? {
    embed: './embed.js'
  }
  : Object.assign({}, Shared.dappsEntry, {
    index: './index.js'
  });

module.exports = {
  cache: !isProd,
  devtool: isProd
    ? false
    : '#source-map',
  context: path.join(__dirname, '../src'),
  entry: entry,
  output: {
    // publicPath: '/',
    path: path.join(__dirname, '../', DEST),
    filename: '[name].[hash:10].js'
  },

  module: {
    rules: [
      {
        test: /\.js$/,
        exclude: /(node_modules)/,
        use: [ 'happypack/loader?id=babel-js' ]
      },
      {
        test: /\.js$/,
        include: /(material-chip-input|ethereumjs-tx)/,
        use: 'babel-loader'
      },
      {
        test: /\.json$/,
        use: [ 'json-loader' ]
      },
      {
        test: /\.ejs$/,
        use: [ 'ejs-loader' ]
      },
      {
        test: /\.html$/,
        use: [
          'file-loader?name=[name].[ext]!extract-loader',
          {
            loader: 'html-loader',
            options: {
              root: path.resolve(__dirname, '../assets/images'),
              attrs: ['img:src', 'link:href']
            }
          }
        ]
      },
      {
        test: /\.md$/,
        use: [
          {
            loader: 'html-loader',
            options: {}
          },
          {
            loader: 'markdown-loader',
            options: {}
          }
        ]
      },
      {
        test: /\.css$/,
        include: [ /src/ ],
        // exclude: [ /src\/dapps/ ],
        loader: (isProd && !isEmbed)
          ? ExtractTextPlugin.extract([
            // 'style-loader',
            'css-loader?modules&sourceMap&importLoaders=1&localIdentName=[name]__[local]___[hash:base64:5]',
            'postcss-loader'
          ])
          : undefined,
        use: (isProd && !isEmbed)
          ? undefined
          : [ 'happypack/loader?id=css' ]
      },

      {
        test: /\.css$/,
        exclude: [ /src/ ],
        use: [ 'style-loader', 'css-loader' ]
      },
      {
        test: /\.(png|jpg)$/,
        use: [ 'file-loader?&name=assets/[name].[hash:10].[ext]' ]
      },
      {
        test: /\.(woff(2)|ttf|eot|otf)(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        use: [ 'file-loader?name=fonts/[name][hash:10].[ext]' ]
      },
      {
        test: /parity-logo-white-no-text\.svg/,
        use: [ 'url-loader' ]
      },
      {
        test: /\.svg(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        exclude: [ /parity-logo-white-no-text\.svg/ ],
        use: [ 'file-loader?name=assets/[name].[hash:10].[ext]' ]
      }
    ],
    noParse: [
      /node_modules\/sinon/
    ]
  },

  resolve: {
    alias: {
      '~/api/local': path.resolve(__dirname, '../src/api/local/localAccountsMiddleware.js'),
      '~': path.resolve(__dirname, '../src'),
      'keythereum': path.resolve(__dirname, '../node_modules/keythereum/dist/keythereum')
    },
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
    const DappsHTMLInjection = DAPPS.filter((dapp) => !dapp.skipBuild).map((dapp) => {
      return new HtmlWebpackPlugin({
        title: dapp.name,
        filename: dapp.url + '.html',
        template: './dapps/index.ejs',
        favicon: FAVICON,
        secure: dapp.secure,
        chunks: [ isProd ? null : 'commons', dapp.url ]
      });
    });

    let plugins = Shared.getPlugins().concat(
      new WebpackErrorNotificationPlugin()
    );

    if (!isEmbed) {
      plugins = [].concat(
        plugins,

        new HtmlWebpackPlugin({
          title: 'Parity',
          filename: 'index.html',
          template: './index.ejs',
          favicon: FAVICON,
          chunks: [
            isProd ? null : 'commons',
            'index'
          ]
        }),

        new ServiceWorkerWebpackPlugin({
          entry: path.join(__dirname, '../src/serviceWorker.js')
        }),

        DappsHTMLInjection,

        new webpack.DllReferencePlugin({
          context: '.',
          manifest: require(`../${DEST}/vendor-manifest.json`)
        }),

        new ScriptExtHtmlWebpackPlugin({
          sync: [ 'commons', 'vendor.js' ],
          defaultAttribute: 'defer'
        }),

        new CopyWebpackPlugin([
          { from: './error_pages.css', to: 'styles.css' },
          { from: './manifest.json', to: 'manifest.json' },
          { from: 'dapps/static' }
        ], {})
      );
    }

    if (isEmbed) {
      plugins.push(
        new HtmlWebpackPlugin({
          title: 'Parity Bar',
          filename: 'embed.html',
          template: './index.ejs',
          favicon: FAVICON,
          chunks: [
            isProd ? null : 'commons',
            'embed'
          ]
        })
      );
    }

    if (!isAnalize && !isProd) {
      const DEST_I18N = path.join(__dirname, '..', DEST, 'i18n');

      plugins.push(
        new ReactIntlAggregatePlugin({
          messagesPattern: DEST_I18N + '/src/**/*.json',
          aggregateOutputDir: DEST_I18N + '/i18n/',
          aggregateFilename: 'en'
        }),

        new webpack.optimize.CommonsChunkPlugin({
          filename: 'commons.[hash:10].js',
          name: 'commons',
          minChunks: 2
        })
      );
    }

    if (isProd) {
      plugins.push(new ExtractTextPlugin({
        filename: 'styles/[name].[hash:10].css',
        allChunks: true
      }));
    }

    return plugins;
  }())
};
