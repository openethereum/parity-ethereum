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

import { bytesToHex, hex2Ascii } from '../../../api/util/format';
import { addCertification } from './actions';

import badgeRegABI from '../../../contracts/abi/badgereg.json';
import certifierABI from '../../../contracts/abi/certifier.json';

const badgeRegAddress = '0xcF5A62987294fd2087252FD812443508528C52bF';
const ZERO = '0x0000000000000000000000000000000000000000000000000000000000000000';
const knownCertifiers = [ 'smsverification' ];

export const fetchCertifier = (api) => {
  const registry = api.newContract(badgeRegABI, badgeRegAddress);
  const cache = {};

  return (name) => {
    if (cache[name]) {
      return Promise.resolve(cache[name]);
    }
    return registry.instance.fromName.call({}, [name])
    .then(([ id, address ]) => {
      return Promise.all([
        registry.instance.meta.call({}, [id, 'TITLE']),
        registry.instance.meta.call({}, [id, 'IMG'])
      ])
        .then(([title, img]) => {
          title = bytesToHex(title);
          title = title === ZERO ? null : hex2Ascii(title);
          if (bytesToHex(img) === ZERO) img = null;

          const data = { address, name, title, icon: img };
          cache[name] = data;
          return data;
        });
    });
  };
};

export const checkIfCertified = (api) => {
  const contracts = {};

  return (certifier, address) => {
    if (!contracts[certifier]) {
      contracts[certifier] = api.newContract(certifierABI, certifier);
    }
    const contract = contracts[certifier];

    return contract.instance.certified.call({}, [address]);
  };
};

export default (api) => {
  const fetch = fetchCertifier(api);
  const check = checkIfCertified(api);

  return (store) => (next) => (action) => {
    if (action.type !== 'fetchCertifications') {
      return next(action);
    }

    const { address } = action;

    knownCertifiers.forEach((name) => {
      fetch(name)
        .then((data) => {
          return check(data.address, address)
            .then((isCertified) => {
              if (isCertified) {
                store.dispatch(addCertification(address, data.name, data.title, data.icon));
              }
            });
        })
        .catch((err) => {
          if (err) {
            console.error(`Failed to check if ${address} certified by ${name}:`, err);
          }
        });
    });
  };
};
