import BigNumber from 'bignumber.js';

const UNITS = ['wei', 'ada', 'babbage', 'shannon', 'szabo', 'finney', 'ether', 'kether', 'mether', 'gether', 'tether'];

export function _getUnitMultiplier (unit) {
  const position = UNITS.indexOf(unit.toLowerCase());

  if (position === -1) {
    throw new Error(`Unknown unit ${unit} passed to wei formatter`);
  }

  return 10 ** (position * 3);
}

export function fromWei (value, unit = 'ether') {
  return new BigNumber(value).div(_getUnitMultiplier(unit));
}

export function toWei (value, unit = 'ether') {
  return new BigNumber(value).mul(_getUnitMultiplier(unit));
}
