import ParamType from './paramType';

describe('abi/spec/paramType/ParamType', () => {
  describe('validateType', () => {
    it('validates address', () => {
      expect(ParamType.validateType('address')).to.be.true;
    });

    it('validates fixedArray', () => {
      expect(ParamType.validateType('fixedArray')).to.be.true;
    });

    it('validates array', () => {
      expect(ParamType.validateType('array')).to.be.true;
    });

    it('validates fixedBytes', () => {
      expect(ParamType.validateType('fixedBytes')).to.be.true;
    });

    it('validates bytes', () => {
      expect(ParamType.validateType('bytes')).to.be.true;
    });

    it('validates bool', () => {
      expect(ParamType.validateType('bool')).to.be.true;
    });

    it('validates int', () => {
      expect(ParamType.validateType('int')).to.be.true;
    });

    it('validates uint', () => {
      expect(ParamType.validateType('uint')).to.be.true;
    });

    it('validates string', () => {
      expect(ParamType.validateType('string')).to.be.true;
    });

    it('throws an error on invalid types', () => {
      expect(() => ParamType.validateType('noMatch')).to.throw(/noMatch/);
    });
  });

  describe('constructor', () => {
    it('throws an error on invalid types', () => {
      expect(() => new ParamType('noMatch')).to.throw(/noMatch/);
    });

    it('sets the type of the object', () => {
      expect((new ParamType('bool', null, 1)).type).to.equal('bool');
    });

    it('sets the subtype of the object', () => {
      expect((new ParamType('array', 'bool', 1)).subtype).to.equal('bool');
    });

    it('sets the length of the object', () => {
      expect((new ParamType('array', 'bool', 1)).length).to.equal(1);
    });
  });
});
