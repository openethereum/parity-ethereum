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

export const checkIfVerified = (contract, account) => {
  return contract.instance.certified.call({}, [account]);
};

export const checkIfRequested = (contract, account) => {
  return new Promise((resolve, reject) => {
    contract.subscribe('Requested', {
      fromBlock: 0, toBlock: 'pending'
    }, (err, logs) => {
      if (err) {
        return reject(err);
      }
      const e = logs.find((l) => {
        return l.type === 'mined' && l.params.who && l.params.who.value === account;
      });
      resolve(e ? e.transactionHash : false);
    });
  });
};
