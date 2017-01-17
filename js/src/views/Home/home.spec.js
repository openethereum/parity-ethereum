// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Home from './';

let api;
let component;
let instance;
let router;

function createApi () {
  api = {
    __id: 'testApi'
  };

  return api;
}

function createRouter () {
  router = {
    push: sinon.stub()
  };

  return router;
}

function render (props = {}) {
  component = shallow(
    <Home { ...props } />,
    {
      context: {
        api: createApi(),
        router: createRouter()
      }
    }
  );
  instance = component.instance();

  return component;
}

describe('views/Home', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('stores', () => {
    describe('webstore', () => {
      it('attaches to a webstore', () => {
        expect(instance.webstore).to.be.ok;
      });

      it('attaches to webstore with context api', () => {
        expect(instance.webstore._api.__id).to.equal(api.__id);
      });
    });
  });

  describe('events', () => {
    beforeEach(() => {
      sinon.spy(instance.webstore, 'gotoUrl');
      sinon.spy(instance.webstore, 'restoreUrl');
      sinon.spy(instance.webstore, 'setNextUrl');
    });

    afterEach(() => {
      instance.webstore.gotoUrl.restore();
      instance.webstore.restoreUrl.restore();
      instance.webstore.setNextUrl.restore();
    });

    describe('onChangeUrl', () => {
      it('performs setNextUrl on store', () => {
        instance.onChangeUrl('123');
        expect(instance.webstore.setNextUrl).to.have.been.calledWith('123');
      });
    });

    describe('onGotoUrl', () => {
      it('performs gotoUrl on store', () => {
        instance.onGotoUrl();
        expect(instance.webstore.gotoUrl).to.have.been.called;
      });

      it('does route navigation when executed', () => {
        instance.onGotoUrl();
        expect(router.push).to.have.been.calledWith('/web');
      });
    });

    describe('onRestoreUrl', () => {
      it('performs restoreUrl on store', () => {
        instance.onRestoreUrl();
        expect(instance.webstore.restoreUrl).to.have.been.called;
      });
    });
  });
});
