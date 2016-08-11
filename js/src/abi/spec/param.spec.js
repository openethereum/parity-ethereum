import Param from './param';

describe('abi/spec/Param', () => {
  describe('constructor', () => {
    const param = new Param('foo', 'uint');

    it('sets the properties', () => {
      expect(param.name).to.equal('foo');
      expect(param.kind.type).to.equal('uint');
    });
  });

  describe('toParams', () => {
    it('maps an array of params', () => {
      const params = Param.toParams([{ name: 'foo', type: 'uint' }]);

      expect(params.length).to.equal(1);
      expect(params[0].name).to.equal('foo');
      expect(params[0].kind.type).to.equal('uint');
    });
  });
});
