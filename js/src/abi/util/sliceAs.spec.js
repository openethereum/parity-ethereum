import { asAddress, asBool, asI32, asU32 } from './sliceAs';

describe('abi/util/sliceAs', () => {
  const MAX_INT = 'ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff';

  describe('asAddress', () => {
    it('correctly returns the last 40 characters', () => {
      const address = '1111111111222222222233333333334444444444';
      expect(asAddress(`000000000000000000000000${address}`)).to.equal(address);
    });
  });

  describe('asBool', () => {
    it('correctly returns true', () => {
      expect(asBool('0000000000000000000000000000000000000000000000000000000000000001')).to.be.true;
    });

    it('correctly returns false', () => {
      expect(asBool('0000000000000000000000000000000000000000000000000000000000000000')).to.be.false;
    });
  });

  describe('asI32', () => {
    it('correctly decodes positive numbers', () => {
      expect(asI32('000000000000000000000000000000000000000000000000000000000000007b').toString()).to.equal('123');
    });

    it('correctly decodes negative numbers', () => {
      expect(asI32('ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff85').toString()).to.equal('-123');
    });
  });

  describe('asU32', () => {
    it('returns a maxium U32', () => {
      expect(asU32(MAX_INT).toString(16)).to.equal(MAX_INT);
    });
  });
});
