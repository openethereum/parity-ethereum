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
import sinon from 'sinon';

import TransactionPendingFormConfirm from './';

const ADDR_NORMAL = '0x0123456789012345678901234567890123456789';
const ADDR_WALLET = '0x1234567890123456789012345678901234567890';
const ADDR_HARDWARE = '0x2345678901234567890123456789012345678901';
const ADDR_SIGN = '0x3456789012345678901234567890123456789012';
const ACCOUNTS = {
  [ADDR_NORMAL]: {
    address: ADDR_NORMAL,
    uuid: ADDR_NORMAL
  },
  [ADDR_WALLET]: {
    address: ADDR_WALLET,
    wallet: true
  },
  [ADDR_HARDWARE]: {
    address: ADDR_HARDWARE,
    hardware: true
  }
};

let component;
let instance;
let onConfirm;

function render (address) {
  onConfirm = sinon.stub();

  component = shallow(
    <TransactionPendingFormConfirm
      account={ ACCOUNTS[address] }
      address={ address }
      onConfirm={ onConfirm }
      isSending={ false }
      dataToSign={ {} }
    />
  );
  instance = component.instance();

  return component;
}

describe('views/Signer/TransactionPendingFormConfirm', () => {
  describe('normal accounts', () => {
    beforeEach(() => {
      render(ADDR_NORMAL);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('does not render the key input', () => {
      expect(instance.renderKeyInput()).to.be.null;
    });

    it('renders the password', () => {
      expect(instance.renderPassword()).not.to.be.null;
    });
  });

  describe('hardware accounts', () => {
    beforeEach(() => {
      render(ADDR_HARDWARE);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('does not render the key input', () => {
      expect(instance.renderKeyInput()).to.be.null;
    });

    it('does not render the password', () => {
      expect(instance.renderPassword()).to.be.null;
    });
  });

  describe('wallet accounts', () => {
    beforeEach(() => {
      render(ADDR_WALLET);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('does not render the key input', () => {
      expect(instance.renderKeyInput()).to.be.null;
    });

    it('renders the password', () => {
      expect(instance.renderPassword()).not.to.be.null;
    });
  });

  describe('signing accounts', () => {
    beforeEach(() => {
      render(ADDR_SIGN);
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    it('renders the key input', () => {
      expect(instance.renderKeyInput()).not.to.be.null;
    });

    it('renders the password', () => {
      expect(instance.renderPassword()).not.to.be.null;
    });

    it('renders the hint', () => {
      expect(instance.renderHint()).to.be.null;
    });
  });
});
