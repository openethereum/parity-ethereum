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

import { bytesToHex, hex2Ascii } from '../api/util/format';

import ABI from './abi/certifier.json';

const ZERO = '0x0000000000000000000000000000000000000000000000000000000000000000';

export default (api, registry) => {
  registry.getContract('badgereg');
  const certifiers = {}; // by name
  const contracts = {}; // by name

  const fetchCertifier = (name) => {
    if (certifiers[name]) {
      return Promise.resolve(certifiers[name]);
    }
    return registry.getContract('badgereg')
      .then((badgeReg) => {
        return badgeReg.instance.fromName.call({}, [name])
        .then(([ id, address ]) => {
          return Promise.all([
            badgeReg.instance.meta.call({}, [id, 'TITLE']),
            badgeReg.instance.meta.call({}, [id, 'IMG'])
          ])
            .then(([ title, img ]) => {
              title = bytesToHex(title);
              title = title === ZERO ? null : hex2Ascii(title);
              if (bytesToHex(img) === ZERO) img = null;

              const data = { address, name, title, icon: img };
              certifiers[name] = data;
              return data;
            });
        });
      });
  };

  const checkIfCertified = (certifier, address) => {
    if (!contracts[certifier]) {
      contracts[certifier] = api.newContract(ABI, certifier);
    }
    const contract = contracts[certifier];

    return contract.instance.certified.call({}, [address]);
  };

  return { fetchCertifier, checkIfCertified };
};
