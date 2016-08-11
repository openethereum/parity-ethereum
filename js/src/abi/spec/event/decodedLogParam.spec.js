import DecodedLogParam from './decodedLogParam';
import ParamType from '../paramType';
import Token from '../../token';

describe('abi/spec/event/DecodedLogParam', () => {
  describe('constructor', () => {
    const pt = new ParamType('bool');
    const tk = new Token('bool');

    it('disallows kind not instanceof ParamType', () => {
      expect(() => new DecodedLogParam('test', 'param')).to.throw(/ParamType/);
    });

    it('disallows token not instanceof Token', () => {
      expect(() => new DecodedLogParam('test', pt, 'token')).to.throw(/Token/);
    });

    it('stores all parameters received', () => {
      const log = new DecodedLogParam('test', pt, tk);

      expect(log.name).to.equal('test');
      expect(log.kind).to.equal(pt);
      expect(log.token).to.equal(tk);
    });
  });
});
