// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import ABI from '../../../contracts/abi/badgereg.json';

const address = '0xcF5A62987294fd2087252FD812443508528C52bF';

export default (api) => {
  const registry = api.newContract(ABI, address);
  const cache = {};

  return (name) => {
    if (cache[name]) {
      return Promise.resolve(cache[name]);
    }
    return registry.instance.fromName.call({}, [name])
    .then((data) => { // id, address, name
      const address = data[1];
      cache[name] = address;
      return address;
    });
  };
};
