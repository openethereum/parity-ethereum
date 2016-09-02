import format from '../api/format';

export const ERRORS = {
  invalidAddress: 'address is an invalid network address',
  duplicateAddress: 'the address is already in your address book',
  invalidChecksum: 'address has failed the checksum formatting',
  invalidName: 'name should not be blank and longer than 2'
};

export function validateAddress (address) {
  let addressError = null;

  if (!address) {
    addressError = ERRORS.invalidAddress;
  } else if (!format.isAddressValid(address)) {
    addressError = ERRORS.invalidAddress;
  } else {
    address = format.toChecksumAddress(address);
  }

  return {
    address,
    addressError
  };
}

export function validateName (name) {
  const nameError = !name || name.length < 2 ? ERRORS.invalidName : null;

  return {
    name,
    nameError
  };
}
