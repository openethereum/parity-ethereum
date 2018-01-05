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

const path = require('path');
const ServiceWorkerWebpackPlugin = require('serviceworker-webpack-plugin');

module.exports = {
  htmlPlugin: {
    favicon: path.resolve(__dirname, './src/assets/images/parity-logo-black-no-text.png'),
    title: 'Parity'
  },
  alias: {
    '~/api/local': path.resolve(__dirname, './src/api/local/localAccountsMiddleware.js'),
    '~': path.resolve(__dirname, './src'),
    'keythereum': path.resolve(__dirname, './node_modules/keythereum/dist/keythereum')
  },
  webpack: {
    plugins: [
      new ServiceWorkerWebpackPlugin({
        entry: path.join(__dirname, './src/serviceWorker.js')
      })
    ],
    modules: [
      /ethereumjs-util/
    ]
  }
};
