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

import { createApi, createStore } from '../createAccount.test.js';

import NewAccount from './';

let api;
let component;
let instance;
let store;

function render () {
  api = createApi();
  store = createStore();
  component = shallow(
    <NewAccount
      createStore={ store }
    />,
    {
      context: { api }
    }
  );
  instance = component.instance();

  return component;
}

describe('modals/CreateAccount/NewAccount', () => {
  beforeEach(() => {
    render();
  });

  it('renders with defaults', () => {
    expect(component).to.be.ok;
  });

  describe('lifecycle', () => {
    describe('componentWillMount', () => {
      beforeEach(() => {
        return instance.componentWillMount();
      });

      it('resets the accounts', () => {
        expect(instance.state.accounts).to.be.null;
      });

      it('resets the initial selected value', () => {
        expect(instance.state.selectedAddress).to.equal('');
      });
    });
  });

  describe('event handlers', () => {
    describe('onChangeIdentity', () => {
      let address;

      beforeEach(() => {
        address = Object.keys(instance.state.accounts)[3];

        sinon.spy(store, 'setAddress');
        sinon.spy(store, 'setPhrase');
        instance.onChangeIdentity({ target: { value: address } });
      });

      afterEach(() => {
        store.setAddress.restore();
        store.setPhrase.restore();
      });

      it('sets the state with the new value', () => {
        expect(instance.state.selectedAddress).to.equal(address);
      });

      it('sets the new address on the store', () => {
        expect(store.setAddress).to.have.been.calledWith(address);
      });

      it('sets the new phrase on the store', () => {
        expect(store.setPhrase).to.have.been.calledWith(instance.state.accounts[address].phrase);
      });
    });

    describe('onEditPassword', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPassword');
        instance.onEditPassword(null, 'test');
      });

      afterEach(() => {
        store.setPassword.restore();
      });

      it('calls into the store', () => {
        expect(store.setPassword).to.have.been.calledWith('test');
      });
    });

    describe('onEditPasswordRepeat', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPasswordRepeat');
        instance.onEditPasswordRepeat(null, 'test');
      });

      afterEach(() => {
        store.setPasswordRepeat.restore();
      });

      it('calls into the store', () => {
        expect(store.setPasswordRepeat).to.have.been.calledWith('test');
      });
    });

    describe('onEditPasswordHint', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPasswordHint');
        instance.onEditPasswordHint(null, 'test');
      });

      afterEach(() => {
        store.setPasswordHint.restore();
      });

      it('calls into the store', () => {
        expect(store.setPasswordHint).to.have.been.calledWith('test');
      });
    });

    describe('onEditAccountName', () => {
      beforeEach(() => {
        sinon.spy(store, 'setName');
        instance.onEditAccountName(null, 'test');
      });

      afterEach(() => {
        store.setName.restore();
      });

      it('calls into the store', () => {
        expect(store.setName).to.have.been.calledWith('test');
      });
    });
  });
});
