import { isAddress, toChecksumAddress } from './address';
import { fromWei, toWei } from './wei';

export default {
  isAddressValid: isAddress,
  fromWei: fromWei,
  toChecksumAddress: toChecksumAddress,
  toWei: toWei
};
