import { isAddress, toChecksumAddress } from './address';
import { fromWei, toWei } from './wei';
import { sha3 } from './sha3';

export default {
  isAddressValid: isAddress,
  fromWei: fromWei,
  toChecksumAddress: toChecksumAddress,
  toWei: toWei,
  sha3: sha3
};
