import { sliceData } from './slice';

describe('abi/util/slice', () => {
  describe('sliceData', () => {
    const slice1 = '131a3afc00d1b1e3461b955e53fc866dcf303b3eb9f4c16f89e388930f48134b';
    const slice2 = '2124768576358735263578356373526387638357635873563586353756358763';

    it('throws an error on mod 64 != 0', () => {
      expect(() => sliceData('123')).to.throw(/sliceData/);
    });

    it('returns an empty array when length === 0', () => {
      expect(sliceData('')).to.deep.equal([]);
    });

    it('returns an array with the slices otherwise', () => {
      const sliced = sliceData(`${slice1}${slice2}`);

      expect(sliced.length).to.equal(2);
      expect(sliced[0]).to.equal(slice1);
      expect(sliced[1]).to.equal(slice2);
    });

    it('removes leading 0x when passed in', () => {
      const sliced = sliceData(`0x${slice1}${slice2}`);

      expect(sliced.length).to.equal(2);
      expect(sliced[0]).to.equal(slice1);
      expect(sliced[1]).to.equal(slice2);
    });
  });
});
