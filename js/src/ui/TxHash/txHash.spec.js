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

import TxHash from './';

const TXHASH = '0xabcdef123454321abcdef';

let api;
let blockNumber;
let callback;
let component;
let instance;

function createApi () {
  blockNumber = new BigNumber(100);
  api = {
    eth: {
      getTransactionByHash: (hash) => {
        return Promise.resolve({
          blockNumber: new BigNumber(100),
          gas: new BigNumber(42000)
        });
      },
      getTransactionReceipt: (hash) => {
        return Promise.resolve({
          blockNumber: new BigNumber(100),
          transactionHash: hash,
          gasUsed: new BigNumber(42000)
        });
      }
    },
    nextBlock: (increment = 1) => {
      blockNumber = blockNumber.plus(increment);
      return callback(null, blockNumber);
    },
    subscribe: (type, _callback) => {
      callback = _callback;
      return callback(null, blockNumber).then(() => {
        return Promise.resolve(1);
      });
    },
    unsubscribe: sinon.stub().resolves(true)
  };

  return api;
}

function createRedux () {
  return {
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
}

function render (props) {
  const baseComponent = shallow(
    <TxHash
      hash={ TXHASH }
      { ...props }
    />,
    { context: { store: createRedux() } }
  );

  component = baseComponent.find('TxHash').shallow({ context: { api: createApi() } });
  instance = component.instance();

  return component;
}

describe('ui/TxHash', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  it('renders the summary', () => {
    expect(component.find('p').find('FormattedMessage').props().id).to.equal('ui.txHash.posted');
  });

  describe('renderConfirmations', () => {
    describe('with no transaction retrieved', () => {
      let child;

      beforeEach(() => {
        child = shallow(instance.renderConfirmations());
      });

      it('renders indeterminate progressbar', () => {
        expect(child.find('LinearProgress[mode="indeterminate"]')).to.have.length(1);
      });

      it('renders waiting text', () => {
        expect(child.find('FormattedMessage').props().id).to.equal('ui.txHash.waiting');
      });
    });

    describe('with transaction retrieved', () => {
      let child;

      beforeEach(() => {
        return instance.componentDidMount().then(() => {
          child = shallow(instance.renderConfirmations());
        });
      });

      it('renders determinate progressbar', () => {
        expect(child.find('LinearProgress[mode="determinate"]')).to.have.length(1);
      });

      it('renders confirmation text', () => {
        expect(child.find('FormattedMessage').props().id).to.equal('ui.txHash.confirmations');
      });

      it('renders with warnings', () => {
        expect(component.find('Warning')).to.have.length.gte(1);
      });

      it('renders with oog warning', () => {
        expect(component.find('Warning').shallow().find('FormattedMessage').prop('id')).to.match(/oog/);
      });
    });
  });
});
