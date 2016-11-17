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

const checkIfTxFailed = (api, tx, gasSent) => {
  return api.pollMethod('eth_getTransactionReceipt', tx)
  .then((receipt) => {
    // TODO: Right now, there's no way to tell wether the EVM code crashed.
    // Because you usually send a bit more gas than estimated (to make sure
    // it gets mined quickly), we transaction probably failed if all the gas
    // has been used up.
    return receipt.gasUsed.eq(gasSent);
  });
};

export default checkIfTxFailed;
