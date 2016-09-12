import bytesToHex from './bytes-array-to-hex';

describe('api/util/bytes-array-to-hex', () => {
  it('correctly converts an empty array', () => {
    expect(bytesToHex([])).to.equal('0x');
  });

  it('correctly converts a non-empty array', () => {
    expect(bytesToHex([0, 15, 16])).to.equal('0x000f10');
  });
});
