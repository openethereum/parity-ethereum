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

import AwaitingDepositStep from './';

const TEST_ADDRESS = '0x123456789123456789123456789123456789';

let component;
let instance;

function render () {
  component = shallow(
    <AwaitingDepositStep
      store={ {
        coinSymbol: 'BTC',
        price: { rate: 0.001, minimum: 0, limit: 1.999 }
      } }
    />
  );
  instance = component.instance();

  return component;
}

describe('views/Account/Shapeshift/AwaitingDepositStep', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('displays waiting for address with empty depositAddress', () => {
    render();
    expect(component.find('FormattedMessage').props().id).to.match(/awaitingConfirmation/);
  });

  it('displays waiting for deposit with non-empty depositAddress', () => {
    render({ depositAddress: 'xyz' });
    expect(component.find('FormattedMessage').first().props().id).to.match(/awaitingDeposit/);
  });

  describe('instance methods', () => {
    describe('renderAddress', () => {
      let address;

      beforeEach(() => {
        address = shallow(instance.renderAddress(TEST_ADDRESS));
      });

      it('renders the address', () => {
        expect(address.text()).to.contain(TEST_ADDRESS);
      });

      describe('CopyToClipboard', () => {
        let copy;

        beforeEach(() => {
          copy = address.find('Connect(CopyToClipboard)');
        });

        it('renders the copy', () => {
          expect(copy.length).to.equal(1);
        });

        it('passes the address', () => {
          expect(copy.props().data).to.equal(TEST_ADDRESS);
        });
      });

      describe('QrCode', () => {
        let qr;

        beforeEach(() => {
          qr = address.find('QrCode');
        });

        it('renders the QrCode', () => {
          expect(qr.length).to.equal(1);
        });

        it('passed the address', () => {
          expect(qr.props().value).to.equal(TEST_ADDRESS);
        });

        describe('protocol link', () => {
          it('does not render a protocol link (unlinked type)', () => {
            expect(address.find('a')).to.have.length(0);
          });

          it('renders protocol link for BTC', () => {
            address = shallow(instance.renderAddress(TEST_ADDRESS, 'BTC'));
            expect(address.find('a').props().href).to.equal(`bitcoin:${TEST_ADDRESS}`);
          });
        });
      });
    });
  });
});
