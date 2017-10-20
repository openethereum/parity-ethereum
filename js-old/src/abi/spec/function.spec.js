// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import Func from './function';
import Param from './param';
import Token from '../token';

describe('abi/spec/Function', () => {
  const inputsArr = [{ name: 'boolin', type: 'bool' }, { name: 'stringin', type: 'string' }];
  const outputsArr = [{ name: 'output', type: 'uint' }];

  const uint = new Param('output', 'uint');
  const bool = new Param('boolin', 'bool');
  const string = new Param('stringin', 'string');
  const inputs = [bool, string];
  const outputs = [uint];

  const func = new Func({
    name: 'test',
    inputs: inputsArr,
    outputs: outputsArr
  });

  describe('constructor', () => {
    it('returns signature correctly if name already contains it', () => {
      const func = new Func({
        name: 'test(bool,string)',
        inputs: inputsArr,
        outputs: outputsArr
      });

      expect(func.name).to.equal('test');
      expect(func.id).to.equal('test(bool,string)');
      expect(func.signature).to.equal('02356205');
    });

    it('stores the parameters as received', () => {
      expect(func.name).to.equal('test');
      expect(func.constant).to.be.false;
      expect(func.inputs).to.deep.equal(inputs);
      expect(func.outputs).to.deep.equal(outputs);
    });

    it('matches empty inputs with []', () => {
      expect(new Func({ name: 'test', outputs: outputsArr }).inputs).to.deep.equal([]);
    });

    it('matches empty outputs with []', () => {
      expect(new Func({ name: 'test', inputs: inputsArr }).outputs).to.deep.equal([]);
    });

    it('sets the method signature', () => {
      expect(new Func({ name: 'baz' }).signature).to.equal('a7916fac');
    });

    it('allows constant functions', () => {
      expect(new Func({ name: 'baz', constant: true }).constant).to.be.true;
    });
  });

  describe('inputParamTypes', () => {
    it('retrieves the input types as received', () => {
      expect(func.inputParamTypes()).to.deep.equal([bool.kind, string.kind]);
    });
  });

  describe('outputParamTypes', () => {
    it('retrieves the output types as received', () => {
      expect(func.outputParamTypes()).to.deep.equal([uint.kind]);
    });
  });

  describe('encodeCall', () => {
    it('encodes the call correctly', () => {
      const result = func.encodeCall([new Token('bool', true), new Token('string', 'jacogr')]);

      expect(result).to.equal('023562050000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000066a61636f67720000000000000000000000000000000000000000000000000000');
    });
  });

  describe('decodeOutput', () => {
    it('decodes the result correctly', () => {
      const result = func.decodeOutput('1111111111111111111111111111111111111111111111111111111111111111');

      expect(result[0].value.toString(16)).to.equal('1111111111111111111111111111111111111111111111111111111111111111');
    });
  });
});
