import React from 'react';
import { shallow } from 'enzyme';
import sinon from 'sinon';

import '../../env-specific/tests';

import Call from './Call';

describe('components/Call', () => {
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
