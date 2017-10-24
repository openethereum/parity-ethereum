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

import Connection from './';

let api;
let component;
let instance;

function createApi () {
  return {
    updateToken: sinon.stub().resolves()
  };
}

function createRedux (isConnected = true, isConnecting = false, needsToken = false) {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        nodeStatus: {
          isConnected,
          isConnecting,
          needsToken
        }
      };
    }
  };
}

function render (store) {
  api = createApi();
  component = shallow(
    <Connection />,
    { context: { store: store || createRedux() } }
  ).find('Connection').shallow({ context: { api } });
  instance = component.instance();

  return component;
}

describe('views/Connection', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('does not render when connected', () => {
    expect(render(createRedux(true)).find('div')).to.have.length(0);
  });

  describe('renderPing', () => {
    it('renders the connecting to node message', () => {
      render();
      const ping = shallow(instance.renderPing());

      expect(ping.find('FormattedMessage').props().id).to.equal('connection.connectingNode');
    });
  });

  describe('renderSigner', () => {
    it('renders the connecting to api message when isConnecting === true', () => {
      render(createRedux(false, true));
      const signer = shallow(instance.renderSigner());

      expect(signer.find('FormattedMessage').props().id).to.equal('connection.connectingAPI');
    });

    it('renders token input when needsToken == true & isConnecting === false', () => {
      render(createRedux(false, false, true));
      const signer = shallow(instance.renderSigner());

      expect(signer.find('FormattedMessage').first().props().id).to.equal('connection.noConnection');
    });
  });

  describe('validateToken', () => {
    beforeEach(() => {
      render();
    });

    it('trims whitespace from passed tokens', () => {
      expect(instance.validateToken(' \t test ing\t  ').token).to.equal('test ing');
    });

    it('validates 4-4-4-4 format', () => {
      expect(instance.validateToken('1234-5678-90ab-cdef').validToken).to.be.true;
    });

    it('validates 4-4-4-4 format (with trimmable whitespace)', () => {
      expect(instance.validateToken(' \t 1234-5678-90ab-cdef \t ').validToken).to.be.true;
    });

    it('validates 4444 format', () => {
      expect(instance.validateToken('1234567890abcdef').validToken).to.be.true;
    });

    it('validates 4444 format (with trimmable whitespace)', () => {
      expect(instance.validateToken(' \t 1234567890abcdef \t ').validToken).to.be.true;
    });
  });

  describe('onChangeToken', () => {
    beforeEach(() => {
      render();
      sinon.spy(instance, 'setToken');
      sinon.spy(instance, 'validateToken');
    });

    afterEach(() => {
      instance.setToken.restore();
      instance.validateToken.restore();
    });

    it('validates tokens passed', () => {
      instance.onChangeToken({ target: { value: 'testing' } });
      expect(instance.validateToken).to.have.been.calledWith('testing');
    });

    it('sets the token on the api when valid', () => {
      instance.onChangeToken({ target: { value: '1234-5678-90ab-cdef' } });
      expect(instance.setToken).to.have.been.called;
    });
  });

  describe('setToken', () => {
    beforeEach(() => {
      render();
    });

    it('calls the api.updateToken', () => {
      component.setState({ token: 'testing' });

      return instance.setToken().then(() => {
        expect(api.updateToken).to.have.been.calledWith('testing');
      });
    });
  });
});
