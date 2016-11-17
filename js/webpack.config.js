
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
const path = require('path');
const postcssImport = require('postcss-import');
const postcssNested = require('postcss-nested');
const postcssVars = require('postcss-simple-vars');
const rucksack = require('rucksack-css');
const webpack = require('webpack');
const WebpackErrorNotificationPlugin = require('webpack-error-notification');
const CopyWebpackPlugin = require('copy-webpack-plugin');

const ENV = process.env.NODE_ENV || 'development';
const isProd = ENV === 'production';
const DEST = process.env.BUILD_DEST || '.build';

module.exports = {
  debug: !isProd,
  cache: !isProd,
  devtool: isProd ? '#eval' : '#cheap-module-eval-source-map',
  context: path.join(__dirname, './src'),
  entry: {
    // dapps
    'basiccoin': ['./dapps/basiccoin.js'],
    'githubhint': ['./dapps/githubhint.js'],
    'registry': ['./dapps/registry.js'],
    'signaturereg': ['./dapps/signaturereg.js'],
    'tokenreg': ['./dapps/tokenreg.js'],
    // app
    'index': ['./index.js']
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
    root: path.join(__dirname, 'node_modules'),
    fallback: path.join(__dirname, 'node_modules'),
    extensions: ['', '.js', '.jsx'],
    unsafeCache: true
  },
  resolveLoaders: {
    root: path.join(__dirname, 'node_modules'),
    fallback: path.join(__dirname, 'node_modules')
  },

  htmlLoader: {
    root: path.resolve(__dirname, 'assets/images'),
    attrs: ['img:src', 'link:href']
  },

  postcss: [
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
  ],
  plugins: (function () {
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
        loaders: isProd ? ['babel'] : [
          'react-hot',
          'babel?cacheDirectory=true'
        ]
      }),
      new CopyWebpackPlugin([{ from: './error_pages.css', to: 'styles.css' }], {}),
      new WebpackErrorNotificationPlugin(),
      new webpack.DefinePlugin({
        'process.env': {
          NODE_ENV: JSON.stringify(ENV),
          RPC_ADDRESS: JSON.stringify(process.env.RPC_ADDRESS),
          PARITY_URL: JSON.stringify(process.env.PARITY_URL),
          LOGGING: JSON.stringify(!isProd)
        }
      }),

      new webpack.DllReferencePlugin({
        context: '.',
        manifest: require(`./${DEST}/vendor-manifest.json`)
      })
    ];

    if (!isProd) {
      plugins.push(
        new webpack.optimize.CommonsChunkPlugin({
          filename: 'commons.js',
          name: 'commons'
        })
      );
    }

    if (isProd) {
      plugins.push(
        new webpack.optimize.CommonsChunkPlugin({
          chunks: ['index'],
          name: 'commons'
        })
      );

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
  }()),
  devServer: {
    contentBase: `./${DEST}`,
    historyApiFallback: false,
    quiet: false,
    hot: !isProd,
    proxy: {
      '/api/*': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        autoRewrite: true
      },
      '/app/*': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
        pathRewrite: {
          '^/app': ''
        }
      },
      '/parity-utils/*': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
        pathRewrite: {
          '^/parity-utils': ''
        }
      },
      '/rpc/*': {
        target: 'http://localhost:8080',
        changeOrigin: true
      }
    }
  }
};
