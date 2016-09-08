import { decodeExtraData } from './decodeExtraData';

describe('MINING SETTINGS', () => {
  describe('EXTRA DATA', () => {
    const str = 'parity/1.0.0/1.0.0-beta2';
    const encoded = '0xd783010000867061726974798b312e302e302d6265746132';

    it('should decode encoded to str', () => {
      expect(decodeExtraData(encoded)).to.equal(str);
    });
  });
});
