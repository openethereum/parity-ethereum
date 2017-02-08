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

import { shallow } from 'enzyme';
import React from 'react';

import RequestOrigin from './';

const context = {
  context: {
    api: {
      transport: {
        sessionHash: '1234'
      }
    }
  }
};

describe('views/Signer/components/RequestOrigin', () => {
  it('renders unknown', () => {
    expect(shallow(
      <RequestOrigin origin={ 'unknown' } />,
      context
    ).text()).to.equal('Requested by unknown');
  });

  it('renders dapps', () => {
    expect(shallow(
      <RequestOrigin origin={ { dapp: 'http://parity.io' } } />,
      context
    ).text()).to.equal('Requested by dapp at http://parity.io');
  });

  it('renders rpc', () => {
    expect(shallow(
      <RequestOrigin origin={ { rpc: '' } } />,
      context
    ).text()).to.equal('Requested by RPC (unidentified)');
  });

  it('renders ipc', () => {
    expect(shallow(
      <RequestOrigin origin={ { ipc: '0x1234' } } />,
      context
    ).text()).to.equal('Requested by IPC session<Connect(IdentityIcon) />');
  });

  it('renders signer', () => {
    expect(shallow(
      <RequestOrigin origin={ { signer: '0x12345' } } />,
      context
    ).text()).to.equal('Requested by UI session<Connect(IdentityIcon) />');

    expect(shallow(
      <RequestOrigin origin={ { signer: '0x1234' } } />,
      context
    ).text()).to.equal('Requested by current tab');
  });
});
