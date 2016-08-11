import BytesTaken from './bytesTaken';

describe('abi/decoder/BytesTaken', () => {
  describe('constructor', () => {
    it('sets the bytes of the object', () => {
      expect((new BytesTaken(1, 2)).bytes).to.equal(1);
    });

    it('sets the newOffset of the object', () => {
      expect((new BytesTaken(3, 4)).newOffset).to.equal(4);
    });
  });
});
