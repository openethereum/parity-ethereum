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

import { padRight, padLeft } from '~/api/util/format';

/**
 * Bytecode of this contract:
 *
 *
pragma solidity ^0.4.10;

contract Querier {
  function Querier
    (address addr, bytes32 sign, uint out_size, uint from, uint limit)
    public
  {
    // The size is 32 bytes for each
    // value, plus 32 bytes for the count
    uint m_size = out_size * limit + 32;

    bytes32 p_return;
    uint p_in;
    uint p_out;

    assembly {
      p_return := mload(0x40)
      mstore(0x40, add(p_return, m_size))

      mstore(p_return, limit)

      p_in := mload(0x40)
      mstore(0x40, add(p_in, 0x24))

      mstore(p_in, sign)

      p_out := add(p_return, 0x20)
    }

    for (uint i = from; i < from + limit; i++) {
      assembly {
        mstore(add(p_in, 0x4), i)
        call(gas, addr, 0x0, p_in, 0x24, p_out, out_size)
        p_out := add(p_out, out_size)
        pop
      }
    }

    assembly {
      return (p_return, m_size)
    }
  }
}
 */

export const bytecode = '0x60606040523415600e57600080fd5b60405160a0806099833981016040528080519190602001805191906020018051919060200180519190602001805191505082810260200160008080806040519350848401604052858452604051602481016040528981529250505060208201855b858701811015609457806004840152878260248560008e5af15090870190600101606f565b8484f300';

export const querier = (api, { address, from, limit }, method) => {
  const { outputs, signature } = method;
  const outLength = 32 * outputs.length;
  const callargs = [
    padLeft(address, 32),
    padRight(signature, 32),
    padLeft(outLength, 32),
    padLeft(from, 32),
    padLeft(limit, 32)
  ].map((v) => v.slice(2)).join('');
  const calldata = bytecode + callargs;

  return api.eth.call({ data: calldata })
    .then((result) => {
      const data = result.slice(2);
      const results = [];

      for (let i = 0; i < limit; i++) {
        const datum = data.substr(2 * (32 + i * outLength), 2 * outLength);
        const decoded = method.decodeOutput('0x' + datum).map((t) => t.value);

        results.push(decoded);
      }

      return results;
    });
};
