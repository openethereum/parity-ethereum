import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase

export function isChecksumValid (_address) {
  const address = _address.replace('0x', '');
  const hash = keccak_256(address.toLowerCase(address));

  for (let n = 0; n < 40; n++) {
    const hashval = parseInt(hash[n], 16);
    const isLower = address[n].toUpperCase() !== address[n];
    const isUpper = address[n].toLowerCase() !== address[n];

    if ((hashval > 7 && isLower) || (hashval <= 7 && isUpper)) {
      return false;
    }
  }

  return true;
}

export function isAddress (address) {
  if (address && address.length === 42) {
    if (!/^(0x)?[0-9a-f]{40}$/i.test(address)) {
      return false;
    } else if (/^(0x)?[0-9a-f]{40}$/.test(address) || /^(0x)?[0-9A-F]{40}$/.test(address)) {
      return true;
    }

    return isChecksumValid(address);
  }

  return false;
}

export function toChecksumAddress (_address) {
  const address = (_address || '').toLowerCase();

  if (!isAddress(address)) {
    return '';
  }

  const hash = keccak_256(address.slice(-40));
  let result = '0x';

  for (let n = 0; n < 40; n++) {
    result = `${result}${parseInt(hash[n], 16) > 7 ? address[n + 2].toUpperCase() : address[n + 2]}`;
  }

  return result;
}
