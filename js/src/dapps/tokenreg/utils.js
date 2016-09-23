import { api } from './parity';

import { eip20 as eip20Abi } from '../../json/';

export const getTokenTotalSupply = (tokenAddress) => {
  return api
    .eth
    .getCode(tokenAddress)
    .then(code => {
      if (!code || /^(0x)?0?$/.test(code)) {
        return null;
      }

      const contract = api.newContract(eip20Abi, tokenAddress);

      return contract
        .instance
        .totalSupply
        .call({}, []);
    });
};
