import { _getUnitMultiplier, fromWei, toWei } from './wei';

describe('api/format/wei', () => {
  describe('_getUnitMultiplier', () => {
    it('returns 10^0 for wei', () => {
      expect(_getUnitMultiplier('wei')).to.equal(10 ** 0);
    });

    it('returns 10^15 for finney', () => {
      expect(_getUnitMultiplier('finney')).to.equal(10 ** 15);
    });

    it('returns 10^18 for ether', () => {
      expect(_getUnitMultiplier('ether')).to.equal(10 ** 18);
    });

    it('throws an error on invalid units', () => {
      expect(() => _getUnitMultiplier('invalid')).to.throw(/passed to wei formatter/);
    });
  });

  describe('fromWei', () => {
    it('formats into ether when nothing specified', () => {
      expect(fromWei('1230000000000000000').toString()).to.equal('1.23');
    });

    it('formats into finney when specified', () => {
      expect(fromWei('1230000000000000000', 'finney').toString()).to.equal('1230');
    });
  });

  describe('toWei', () => {
    it('formats from ether when nothing specified', () => {
      expect(toWei(1.23).toString()).to.equal('1230000000000000000');
    });

    it('formats from finney when specified', () => {
      expect(toWei(1230, 'finney').toString()).to.equal('1230000000000000000');
    });
  });
});
