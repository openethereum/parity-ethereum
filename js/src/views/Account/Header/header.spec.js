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

import { ETH_TOKEN } from '@parity/shared/util/tokens';

import Header from './';

const ACCOUNT = {
  address: '0x0123456789012345678901234567890123456789',
  meta: {
    description: 'the description',
    tags: ['taga', 'tagb']
  },
  uuid: '0xabcdef'
};
const subscriptions = {};

let component;
let instance;

const api = {
  subscribe: (method, callback) => {
    subscriptions[method] = (subscriptions[method] || []).concat(callback);
    return Promise.resolve(0);
  },
  eth: {
    getTransactionCount: () => Promise.resolve(new BigNumber(1))
  }
};

function reduxStore () {
  const getState = () => ({
    balances: {},
    tokens: {
      [ETH_TOKEN.id]: ETH_TOKEN
    }
  });

  return {
    getState,
    dispatch: () => null,
    subscribe: () => null
  };
}

function render (props = {}) {
  if (props && !props.account) {
    props.account = ACCOUNT;
  }

  component = shallow(
    <Header { ...props } />,
    { context: { api } }
  );
  instance = component.instance();

  return component;
}

describe('views/Account/Header', () => {
  describe('rendering', () => {
    it('renders defaults', () => {
      expect(render()).to.be.ok;
    });

    it('renders null with no account', () => {
      expect(render(null).find('div')).to.have.length(0);
    });

    it('renders when no account meta', () => {
      expect(render({ account: { address: ACCOUNT.address } })).to.be.ok;
    });

    it('renders when no account description', () => {
      expect(render({ account: { address: ACCOUNT.address, meta: { tags: [] } } })).to.be.ok;
    });

    it('renders when no account tags', () => {
      expect(render({ account: { address: ACCOUNT.address, meta: { description: 'something' } } })).to.be.ok;
    });

    describe('sections', () => {
      describe('Balance', () => {
        let balance;

        beforeEach(() => {
          render();
          balance = component.find('Connect(Balance)')
            .shallow({ context: { store: reduxStore() } });
        });

        it('renders', () => {
          expect(balance).to.have.length(1);
        });

        it('passes the account', () => {
          expect(balance.props().address).to.deep.equal(ACCOUNT.address);
        });
      });

      describe('Certifications', () => {
        let certs;

        beforeEach(() => {
          render();
          certs = component.find('Connect(Certifications)');
        });

        it('renders', () => {
          expect(certs).to.have.length(1);
        });

        it('passes the address', () => {
          expect(certs.props().address).to.deep.equal(ACCOUNT.address);
        });
      });

      describe('IdentityIcon', () => {
        let icon;

        beforeEach(() => {
          render();
          icon = component.find('IdentityIcon');
        });

        it('renders', () => {
          expect(icon).to.have.length(1);
        });

        it('passes the address', () => {
          expect(icon.props().address).to.deep.equal(ACCOUNT.address);
        });
      });

      describe('QrCode', () => {
        let qr;

        beforeEach(() => {
          render();
          qr = component.find('QrCode');
        });

        it('renders', () => {
          expect(qr).to.have.length(1);
        });

        it('passes the address', () => {
          expect(qr.props().value).to.deep.equal(ACCOUNT.address);
        });
      });

      describe('Tags', () => {
        let tags;

        beforeEach(() => {
          render();
          tags = component.find('Tags');
        });

        it('renders', () => {
          expect(tags).to.have.length(1);
        });

        it('passes the tags', () => {
          expect(tags.props().tags).to.deep.equal(ACCOUNT.meta.tags);
        });
      });
    });
  });

  describe('renderName', () => {
    it('renders null with hideName', () => {
      render({ hideName: true });
      expect(instance.renderName()).to.be.null;
    });

    it('renders the name', () => {
      render();
      expect(instance.renderName()).not.to.be.null;
    });

    it('renders when no address specified', () => {
      render({ account: {} });
      expect(instance.renderName()).to.be.ok;
    });
  });

  describe('renderTxCount', () => {
    it('renders null when txCount is null', () => {
      render();
      expect(instance.renderTxCount()).to.be.null;
    });

    it('renders null when contract', () => {
      render({ isContract: true });

      subscriptions['eth_blockNumber'].forEach((callback) => {
        callback();

        setTimeout(() => {
          expect(instance.renderTxCount()).to.be.null;
        });
      });
    });

    it('renders the tx count', () => {
      render();

      subscriptions['eth_blockNumber'].forEach((callback) => {
        callback();

        setTimeout(() => {
          expect(instance.renderTxCount()).not.to.be.null;
        });
      });
    });
  });

  describe('renderUuid', () => {
    it('renders null with no uuid', () => {
      render({ account: Object.assign({}, ACCOUNT, { uuid: null }) });
      expect(instance.renderUuid()).to.be.null;
    });

    it('renders the uuid', () => {
      render();
      expect(instance.renderUuid()).not.to.be.null;
    });
  });
});
