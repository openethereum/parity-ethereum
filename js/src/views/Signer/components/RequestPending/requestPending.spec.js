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

import BigNumber from 'bignumber.js';
import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import RequestPending from './';

const ADDRESS = '0x1234567890123456789012345678901234567890';
const TRANSACTION = {
  from: ADDRESS,
  gas: new BigNumber(21000),
  gasPrice: new BigNumber(20000000),
  value: new BigNumber(1)
};
const PAYLOAD_SENDTX = {
  sendTransaction: TRANSACTION
};
const PAYLOAD_SIGN = {
  sign: {
    address: ADDRESS,
    data: 'testing'
  }
};
const PAYLOAD_SIGNTX = {
  signTransaction: TRANSACTION
};

let component;
let onConfirm;
let onReject;

function render (payload) {
  onConfirm = sinon.stub();
  onReject = sinon.stub();

  component = shallow(
    <RequestPending
      date={ new Date() }
      gasLimit={ new BigNumber(100000) }
      id={ new BigNumber(123) }
      isSending={ false }
      netVersion='42'
      onConfirm={ onConfirm }
      onReject={ onReject }
      origin={ {} }
      payload={ payload }
      store={ {} }
    />
  );

  return component;
}

describe('views/Signer/RequestPending', () => {
  describe('sendTransaction', () => {
    beforeEach(() => {
      render(PAYLOAD_SENDTX);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('renders TransactionPending component', () => {
      expect(component.find('Connect(TransactionPending)')).to.have.length(1);
    });
  });

  describe('sign', () => {
    beforeEach(() => {
      render(PAYLOAD_SIGN);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('renders SignRequest component', () => {
      expect(component.find('SignRequest')).to.have.length(1);
    });
  });

  describe('signTransaction', () => {
    beforeEach(() => {
      render(PAYLOAD_SIGNTX);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('renders TransactionPending component', () => {
      expect(component.find('Connect(TransactionPending)')).to.have.length(1);
    });
  });
});
