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

import React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import Api from '~/api';

import TxList from './txList';

const api = new Api({ execute: sinon.stub() });

const STORE = {
  dispatch: sinon.stub(),
  subscribe: sinon.stub(),
  getState: () => {
    return {
      nodeStatus: {
        netVersion: '42'
      }
    };
  }
};

function render (props) {
  return shallow(
    <TxList
      store={ STORE }
      { ...props }
    />,
    { context: { api } }
  );
}

describe('ui/TxList', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(render({ address: '0x123', hashes: [] })).to.be.ok;
    });
  });
});
