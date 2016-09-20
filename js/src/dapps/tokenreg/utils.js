import { api } from './parity';

import basicTokenAbi from './abi/basictoken.json';

export const getTokenTotalSupply = (tokenAddress) => {
  return api
    .eth
    .getCode(tokenAddress)
    .then(code => {
      if (!code || /^(0x)?0?$/.test(code)) {
        return null;
      }

      const contract = api.newContract(basicTokenAbi, tokenAddress);

      return contract
        .instance
        .totalSupply
        .call({}, []);
    });
};
