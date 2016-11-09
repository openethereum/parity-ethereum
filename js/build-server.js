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
// test only
/**
 * Run `PARITY_URL="127.0.0.1:8180" NODE_ENV="production" npm run build`
 * to build the project ; use this server to test that the minifed
 * version is working (this is a simple proxy server)
 */

var express = require('express');
var proxy = require('http-proxy-middleware');

var app = express();
var wsProxy = proxy('ws://127.0.0.1:8180', { changeOrigin: true });

app.use(express.static('.build'));

app.use('/api/*', proxy({
  target: 'http://127.0.0.1:8080',
  changeOrigin: true
}));

app.use('/app/*', proxy({
  target: 'http://127.0.0.1:8080',
  changeOrigin: true,
  pathRewrite: {
    '^/app': ''
  }
}));

app.use('/parity-utils/*', proxy({
  target: 'http://127.0.0.1:3000',
  changeOrigin: true,
  pathRewrite: {
    '^/parity-utils': ''
  }
}));

app.use('/rpc/*', proxy({
  target: 'http://127.0.0.1:8080',
  changeOrigin: true
}));

app.use(wsProxy);

var server = app.listen(3000);

server.on('upgrade', wsProxy.upgrade);
