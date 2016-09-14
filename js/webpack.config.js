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

var rucksack = require('rucksack-css');
var webpack = require('webpack');
var path = require('path');

var ExtractTextPlugin = require('extract-text-webpack-plugin');
var WebpackErrorNotificationPlugin = require('webpack-error-notification');

var ENV = process.env.NODE_ENV || 'development';
var isProd = ENV === 'production';

module.exports = {
  debug: !isProd,
  cache: !isProd,
  devtool: isProd ? '#eval' : '#cheap-module-eval-source-map',
  context: path.join(__dirname, './src'),
  entry: {
    // dapps
    'gavcoin': ['./dapps/gavcoin.js'],
    'registry': ['./dapps/registry.js'],
    'tokenreg': ['./dapps/tokenreg.js'],
    // library
    'parity': ['./parity.js'],
    // 'web3': ['./web3.js'],
    // app
    'index': ['./index.js']
  },
  output: {
    path: path.join(__dirname, 'build'),
    filename: '[name].js'
  },
  module: {
    loaders: [
      {
        test: /\.js$/,
        exclude: /node_modules/,
        loaders: isProd ? ['babel'] : [
          'react-hot',
          'babel'
        ]
      },
      {
        test: /\.js$/,
        include: /dapps-react-components/,
        loaders: isProd ? ['babel'] : [
          'react-hot',
          'babel'
        ]
      },
      {
        test: /\.json$/,
        loaders: ['json']
      },
      {
        test: /\.html$/,
        loader: 'file?name=[name].[ext]'
      },
      {
        test: /\.css$/,
        include: [/src/],
        loaders: [
          'style',
          'css?modules&sourceMap&importLoaders=1&localIdentName=[name]__[local]___[hash:base64:5]',
          'postcss'
        ]
      },
      {
        test: /\.css$/,
        exclude: [/src/],
        loader: 'style!css'
      },
      {
        test: /\.less$/,
        loaders: [
          'style',
          'css',
          'less'
        ]
      },
      {
        test: /\.(png|jpg|)$/,
        loader: 'file-loader'
      },
      {
        test: /\.woff(2)?(\?v=[0-9]\.[0-9]\.[0-9])?$/,
        loader: 'url-loader?limit=10000&mimetype=application/font-woff'
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
  postcss: [
    rucksack({
      autoprefixer: true
    })
  ],
  plugins: (function () {
    var plugins = [
      new ExtractTextPlugin('[name].css'),
      new WebpackErrorNotificationPlugin(),
      // TODO [todr] paths in dapp-styles is hardcoded for meteor, we need to rewrite it here
      // TODO [jacogr] this shit needs to go, e.g. dapp-styles
      new webpack.NormalModuleReplacementPlugin(
        /ethereum_dapp-styles/,
        function (a) {
          a.request = a.request.replace('./packages/ethereum_dapp-styles', '.');
          a.request = a.request.replace('./lib/packages/ethereum_dapp-styles', '.');
          return a;
        }
      ),
      new webpack.DefinePlugin({
        'process.env': {
          NODE_ENV: JSON.stringify(ENV),
          RPC_ADDRESS: JSON.stringify(process.env.RPC_ADDRESS),
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
  }()),
  devServer: {
    contentBase: './src',
    historyApiFallback: false,
    quiet: false,
    hot: !isProd,
    proxy: {
      '/rpc/*': {
        target: 'http://localhost:8080',
        changeOrigin: true
      },
      '/api/ping': {
        target: 'http://127.0.0.1:8080/index.html',
        changeOrigin: true
      }
    }
  }
};
