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

// NOTE: Keep 'isTestnet' for backwards library compatibility
const getUrlPrefix = (isTestnet = false, netVersion = '0', defaultPrefix = '') => {
  if (isTestnet) {
    return 'ropsten.';
  }

  switch (netVersion) {
    case '1':
      return defaultPrefix;

    case '3':
      return 'ropsten.';

    case '4':
      return 'rinkeby.';

    case '42':
      return 'kovan.';

    default:
      return 'testnet.';
  }
};

export const url = (isTestnet = false, netVersion = '0', defaultPrefix = '') => {
  return `https://${getUrlPrefix(isTestnet, netVersion, defaultPrefix)}etherscan.io`;
};

export const txLink = (hash, isTestnet = false, netVersion = '0') => {
  return `${url(isTestnet, netVersion)}/tx/${hash}`;
};

export const addressLink = (address, isTestnet = false, netVersion = '0') => {
  return `${url(isTestnet, netVersion)}/address/${address}`;
};

export const apiLink = (query, isTestnet = false, netVersion = '0') => {
  return `${url(isTestnet, netVersion, 'api.')}/api?${query}`;
};
