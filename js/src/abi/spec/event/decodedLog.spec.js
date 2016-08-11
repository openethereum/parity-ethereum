import DecodedLog from './decodedLog';

const log = new DecodedLog('someParams', 'someAddress');

describe('abi/spec/event/DecodedLog', () => {
  describe('constructor', () => {
    it('sets internal state', () => {
      expect(log.params).to.equal('someParams');
      expect(log.address).to.equal('someAddress');
    });
  });
});
