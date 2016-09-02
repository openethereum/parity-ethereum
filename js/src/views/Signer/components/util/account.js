import { BASE_LINK_ACCOUNT_MORDEN, BASE_LINK_ACCOUNT_HOMESTEAD } from '../constants/constants';

export const getAccountLink = _getAccountLink;

function _getAccountLink (address, chain) {
  const isTestNet = chain === 'morden' || chain === 'testnet';
  const base = isTestNet ? BASE_LINK_ACCOUNT_MORDEN : BASE_LINK_ACCOUNT_HOMESTEAD;
  return base + address;
}
