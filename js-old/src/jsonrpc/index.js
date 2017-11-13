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

import db from './interfaces/db';
import eth from './interfaces/eth';
import net from './interfaces/net';
import parity from './interfaces/parity';
import personal from './interfaces/personal';
import shh from './interfaces/shh';
import signer from './interfaces/signer';
import trace from './interfaces/trace';
import web3 from './interfaces/web3';

export default {
  db,
  eth,
  parity,
  net,
  personal,
  shh,
  signer,
  trace,
  web3
};
