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

import Urls from './';

const NEXT_URL = 'http://somewhere.next';

let component;
let instance;
let router;
let store;

function createRouter () {
  router = {
    push: sinon.stub()
  };

  return router;
}

function createStore () {
  store = {
    history: [],
    gotoUrl: sinon.stub(),
    restoreUrl: sinon.stub(),
    setNextUrl: sinon.stub(),
    nextUrl: NEXT_URL
  };

  return store;
}

function render () {
  component = shallow(
    <Urls
      extensionStore={ { hasExtension: false } }
      store={ createStore() }
    />,
    {
      context: {
        router: createRouter()
      }
    }
  );
  instance = component.instance();

  return component;
}

describe('views/Home/Urls', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('input', () => {
    let input;

    beforeEach(() => {
      input = component.find('DappUrlInput');
    });

    it('renders the input cmponent', () => {
      expect(input.length).to.equal(1);
    });

    it('passes nextUrl as url', () => {
      expect(input.props().url).to.equal(NEXT_URL);
    });
  });

  describe('events', () => {
    describe('onChangeUrl', () => {
      it('performs setNextUrl on store', () => {
        instance.onChangeUrl('123');
        expect(store.setNextUrl).to.have.been.calledWith('123');
      });
    });

    describe('onGotoUrl', () => {
      it('performs gotoUrl on store', () => {
        instance.onGotoUrl();
        expect(store.gotoUrl).to.have.been.called;
      });

      it('passed the URL when provided', () => {
        instance.onGotoUrl('http://example.com');
        expect(store.gotoUrl).to.have.been.calledWith('http://example.com');
      });

      it('does route navigation when executed', () => {
        instance.onGotoUrl();
        expect(router.push).to.have.been.calledWith('/web');
      });
    });

    describe('onRestoreUrl', () => {
      it('performs restoreUrl on store', () => {
        instance.onRestoreUrl();
        expect(store.restoreUrl).to.have.been.called;
      });
    });
  });
});
