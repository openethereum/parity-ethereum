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

import { createStore } from '../createAccount.test.js';

import RecoveryPhrase from './';

let component;
let instance;
let store;

function render () {
  store = createStore();
  component = shallow(
    <RecoveryPhrase
      createStore={ store }
    />
  );
  instance = component.instance();

  return component;
}

describe('modals/CreateAccount/RecoveryPhrase', () => {
  beforeEach(() => {
    render();
  });

  it('renders with defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event handlers', () => {
    describe('onEditName', () => {
      beforeEach(() => {
        sinon.spy(store, 'setName');
        instance.onEditName(null, 'testValue');
      });

      afterEach(() => {
        store.setName.restore();
      });

      it('calls into the store', () => {
        expect(store.setName).to.have.been.calledWith('testValue');
      });
    });

    describe('onEditPhrase', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPhrase');
        instance.onEditPhrase(null, 'testValue');
      });

      afterEach(() => {
        store.setPhrase.restore();
      });

      it('calls into the store', () => {
        expect(store.setPhrase).to.have.been.calledWith('testValue');
      });
    });

    describe('onEditPassword', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPassword');
        instance.onEditPassword(null, 'testValue');
      });

      afterEach(() => {
        store.setPassword.restore();
      });

      it('calls into the store', () => {
        expect(store.setPassword).to.have.been.calledWith('testValue');
      });
    });

    describe('onEditPasswordRepeat', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPasswordRepeat');
        instance.onEditPasswordRepeat(null, 'testValue');
      });

      afterEach(() => {
        store.setPasswordRepeat.restore();
      });

      it('calls into the store', () => {
        expect(store.setPasswordRepeat).to.have.been.calledWith('testValue');
      });
    });

    describe('onEditPasswordHint', () => {
      beforeEach(() => {
        sinon.spy(store, 'setPasswordHint');
        instance.onEditPasswordHint(null, 'testValue');
      });

      afterEach(() => {
        store.setPasswordHint.restore();
      });

      it('calls into the store', () => {
        expect(store.setPasswordHint).to.have.been.calledWith('testValue');
      });
    });

    describe('onToggleWindowsPhrase', () => {
      beforeEach(() => {
        sinon.spy(store, 'setWindowsPhrase');
        instance.onToggleWindowsPhrase();
      });

      afterEach(() => {
        store.setWindowsPhrase.restore();
      });

      it('calls into the store', () => {
        expect(store.setWindowsPhrase).to.have.been.calledWith(true);
      });
    });
  });
});
