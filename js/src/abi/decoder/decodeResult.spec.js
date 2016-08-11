import DecodeResult from './decodeResult';

describe('abi/decoder/DecodeResult', () => {
  describe('constructor', () => {
    it('sets the token of the object', () => {
      expect((new DecodeResult('token', 2)).token).to.equal('token');
    });

    it('sets the newOffset of the object', () => {
      expect((new DecodeResult('baz', 4)).newOffset).to.equal(4);
    });
  });
});
