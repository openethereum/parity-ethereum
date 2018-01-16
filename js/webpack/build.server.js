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
// test only
/**
 * Run `DAPPS_URL="/" PARITY_URL="127.0.0.1:8546" NODE_ENV="production" npm run build`
 * to build the project ; use this server to test that the minifed
 * version is working (this is a simple proxy server)
 */

var express = require('express');

var Shared = require('./shared');

var app = express();

Shared.addProxies(app);

app.use(express.static('.build'));

var server = app.listen(process.env.PORT || 3000, function () {
  console.log('Listening on port', server.address().port);
});
