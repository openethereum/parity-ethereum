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

import React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import '../../../../environment/tests';

import Call from './Call';

describe('views/Status/components/Call', () => {
  const call = { callIdx: 123, callNo: 456, name: 'eth_call', params: [{ name: '123' }], response: '' };
  const element = 'dummyElement';

  let rendered;
  let instance;
  let setActiveCall = sinon.stub();

  beforeEach(() => {
    rendered = shallow(
      <Call
        call={ call }
        setActiveCall={ setActiveCall }
      />
    );
    instance = rendered.instance();
  });

  describe('rendering', () => {
    it('renders the component', () => {
      expect(rendered).to.be.ok;
      expect(rendered).to.have.exactly(1).descendants(`div[data-test="Call-call-${call.callNo}"]`);
    });

    it('adds onMouseEnter to setActiveElement', () => {
      expect(rendered.find('div').first()).to.have.prop('onMouseEnter', instance.setActiveCall);
    });
  });

  describe('actions', () => {
    it('sets the element via setElement', () => {
      expect(instance.element).to.not.be.ok;
      instance.setElement(element);
      expect(instance.element).to.equal(element);
    });

    it('calls parent setActive call on setActiveCall', () => {
      instance.setElement(element);
      instance.setActiveCall();

      expect(setActiveCall).to.be.calledWith(call, element);
    });
  });

  describe('utility', () => {
    describe('.formatParams', () => {
      it('correctly returns a single parameter', () => {
        expect(instance.formatParams([1])).to.equal('1');
      });

      it('correctly joins 2 parameters', () => {
        expect(instance.formatParams([1, 2])).to.equal('1, 2');
      });

      it('stringifies a string object', () => {
        expect(instance.formatParams(['1'])).to.equal('"1"');
      });

      it('stringifies an object object', () => {
        expect(instance.formatParams([{ name: '1' }])).to.equal('{"name":"1"}');
      });

      it('skips an undefined value', () => {
        expect(instance.formatParams(['1', undefined, 3])).to.equal('"1", , 3');
      });
    });
  });
});
