import sinon from 'sinon';

import { isArray, isFunction, isInstanceOf, isString } from './types';
import Eth from '../rpc/eth';

describe('api/util/types', () => {
  describe('isArray', () => {
    it('correctly identifies null as false', () => {
      expect(isArray(null)).to.be.false;
    });

    it('correctly identifies empty array as true', () => {
      expect(isArray([])).to.be.true;
    });

    it('correctly identifies array as true', () => {
      expect(isArray([1, 2, 3])).to.be.true;
    });
  });

  describe('isFunction', () => {
    it('correctly identifies null as false', () => {
      expect(isFunction(null)).to.be.false;
    });

    it('correctly identifies function as true', () => {
      expect(isFunction(sinon.stub())).to.be.true;
    });
  });

  describe('isInstanceOf', () => {
    it('correctly identifies build-in instanceof', () => {
      expect(isInstanceOf(new String('123'), String)).to.be.true; // eslint-disable-line no-new-wrappers
    });

    it('correctly identifies own instanceof', () => {
      expect(isInstanceOf(new Eth({}), Eth)).to.be.true;
    });

    it('correctly reports false for own', () => {
      expect(isInstanceOf({}, Eth)).to.be.false;
    });
  });

  describe('isString', () => {
    it('correctly identifies empty string as string', () => {
      expect(isString('')).to.be.true;
    });

    it('correctly identifies string as string', () => {
      expect(isString('123')).to.be.true;
    });
  });
});
