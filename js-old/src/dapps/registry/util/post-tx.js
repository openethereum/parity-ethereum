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

const postTx = (api, method, opt = {}, values = []) => {
  opt = Object.assign({}, opt);

  return method.estimateGas(opt, values)
    .then((gas) => {
      opt.gas = gas.mul(1.2).toFixed(0);
      return method.postTransaction(opt, values);
    })
    .then((reqId) => {
      return api.pollMethod('parity_checkRequest', reqId);
    });
};

export default postTx;
