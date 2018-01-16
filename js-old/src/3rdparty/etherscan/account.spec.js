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

const etherscan = require('./');

const TESTADDR = '0xbf885e2b55c6bcc84556a3c5f07d3040833c8d00';

describe.skip('etherscan/account', function () {
  this.timeout(60 * 1000);

  const checkBalance = function (balance, addr) {
    expect(balance).to.be.ok;
    expect(balance.account).to.equal(addr);
    expect(balance.balance).to.be.ok;
  };

  it('retrieves an account balance', () => {
    return etherscan.account
      .balance(TESTADDR)
      .then((balance) => {
        checkBalance(balance, TESTADDR);
      });
  });

  it('retrieves multi account balances', () => {
    const addresses = ['0xde0b295669a9fd93d5f28d9ec85e40f4cb697bae', TESTADDR];

    return etherscan.account
      .balances(addresses)
      .then((balances) => {
        expect(balances).to.be.ok;
        expect(balances.length).to.equal(2);
        balances.forEach((balance, idx) => {
          checkBalance(balance, addresses[idx]);
        });
      });
  });

  describe('transactions', () => {
    it('retrieves a list of transactions (default)', () => {
      return etherscan.account
        .transactions(TESTADDR)
        .then((transactions) => {
          expect(transactions).to.be.ok;
          expect(transactions.length).to.equal(25);
        });
    });

    it('retrieves a list of transactions (page 1)', () => {
      return etherscan.account
        .transactions(TESTADDR, 1)
        .then((transactions) => {
          expect(transactions).to.be.ok;
          expect(transactions.length).to.equal(25);
        });
    });
  });
});
