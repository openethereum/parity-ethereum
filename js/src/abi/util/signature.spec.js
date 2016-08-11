import { eventSignature, methodSignature } from './signature';

describe('abi/util/signature', () => {
  describe('eventSignature', () => {
    it('encodes signature baz() correctly', () => {
      expect(eventSignature('baz', []))
        .to.equal('a7916fac4f538170f7cd12c148552e2cba9fcd72329a2dd5b07a6fa906488ddf');
    });

    it('encodes signature baz(uint32) correctly', () => {
      expect(eventSignature('baz', [{ type: 'uint', length: 32 }]))
        .to.equal('7d68785e8fc871be024b75964bd86d093511d4bc2dc7cf7bea32c48a0efaecb1');
    });

    it('encodes signature baz(uint32, bool) correctly', () => {
      expect(eventSignature('baz', [{ type: 'uint', length: 32 }, { type: 'bool' }]))
        .to.equal('cdcd77c0992ec5bbfc459984220f8c45084cc24d9b6efed1fae540db8de801d2');
    });

    it('encodes no-name signature correctly as ()', () => {
      expect(eventSignature(undefined, []))
        .to.equal('861731d50c3880a2ca1994d5ec287b94b2f4bd832a67d3e41c08177bdd5674fe');
    });

    it('encodes no-params signature correctly as ()', () => {
      expect(eventSignature(undefined, undefined))
        .to.equal('861731d50c3880a2ca1994d5ec287b94b2f4bd832a67d3e41c08177bdd5674fe');
    });
  });

  describe('methodSignature', () => {
    it('encodes signature baz() correctly', () => {
      expect(methodSignature('baz', [])).to.equal('a7916fac');
    });

    it('encodes signature baz(uint32) correctly', () => {
      expect(methodSignature('baz', [{ type: 'uint', length: 32 }])).to.equal('7d68785e');
    });

    it('encodes signature baz(uint32, bool) correctly', () => {
      expect(methodSignature('baz', [{ type: 'uint', length: 32 }, { type: 'bool' }])).to.equal('cdcd77c0');
    });

    it('encodes no-name signature correctly as ()', () => {
      expect(methodSignature(undefined, [])).to.equal('861731d5');
    });

    it('encodes no-params signature correctly as ()', () => {
      expect(methodSignature(undefined, undefined)).to.equal('861731d5');
    });
  });
});
