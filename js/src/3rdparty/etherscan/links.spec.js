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

const { url, txLink, addressLink, apiLink } = require('./links');

describe('etherscan/links', function () {
  it('builds link with a prefix', () => {
    expect(url(false, '1', 'api.')).to.be.equal('https://api.etherscan.io');
  });

  it('builds link to main network', () => {
    expect(url(false, '1')).to.be.equal('https://etherscan.io');
  });

  it('builds link to ropsten', () => {
    expect(url(false, '3')).to.be.equal('https://ropsten.etherscan.io');
    expect(url(true)).to.be.equal('https://ropsten.etherscan.io');
  });

  it('builds link to kovan', () => {
    expect(url(false, '42')).to.be.equal('https://kovan.etherscan.io');
  });

  it('builds link to rinkeby', () => {
    expect(url(false, '4')).to.be.equal('https://rinkeby.etherscan.io');
  });

  it('builds link to the testnet selector for unknown networks', () => {
    expect(url(false, '10042')).to.be.equal('https://testnet.etherscan.io');
    expect(url(false, '51224')).to.be.equal('https://testnet.etherscan.io');
  });

  it('builds transaction link', () => {
    expect(txLink('aTxHash', false, '1')).to.be.equal('https://etherscan.io/tx/aTxHash');
  });

  it('builds address link', () => {
    expect(addressLink('anAddress', false, '1')).to.be.equal('https://etherscan.io/address/anAddress');
  });

  it('builds api link', () => {
    expect(apiLink('answer=42', false, '1')).to.be.equal('https://api.etherscan.io/api?answer=42');
  });
});
