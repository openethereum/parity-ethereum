import { sha3 } from './sha3';

describe('api/format/sha3', () => {
  describe('sha3', () => {
    it('constructs a correct sha3 value', () => {
      expect(sha3('jacogr')).to.equal('0x2f4ff4b5a87abbd2edfed699db48a97744e028c7f7ce36444d40d29d792aa4dc');
    });
  });
});
