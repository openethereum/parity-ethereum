import { isArray, isString, isInstanceOf } from './types';
import Token from '../token';

describe('abi/util/types', () => {
  describe('isArray', () => {
    it('correctly identifies empty arrays as Array', () => {
      expect(isArray([])).to.be.true;
    });

    it('correctly identifies non-empty arrays as Array', () => {
      expect(isArray([1, 2, 3])).to.be.true;
    });

    it('correctly identifies strings as non-Array', () => {
      expect(isArray('not an array')).to.be.false;
    });

    it('correctly identifies objects as non-Array', () => {
      expect(isArray({})).to.be.false;
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

  describe('isInstanceOf', () => {
    it('correctly identifies build-in instanceof', () => {
      expect(isInstanceOf(new String('123'), String)).to.be.true; // eslint-disable-line no-new-wrappers
    });

    it('correctly identifies own instanceof', () => {
      expect(isInstanceOf(new Token('int', 123), Token)).to.be.true;
    });

    it('correctly reports false for own', () => {
      expect(isInstanceOf({ type: 'int' }, Token)).to.be.false;
    });
  });
});
