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

import ABI from '../../../contracts/abi/certifier.json';

export default (api) => {
  const contracts = {};

  return (certifier, address) => {
    if (!contracts[certifier]) {
      contracts[certifier] = api.newContract(ABI, certifier);
    }
    const contract = contracts[certifier];

    return contract.instance.certified.call({}, [address]);
  };
};
