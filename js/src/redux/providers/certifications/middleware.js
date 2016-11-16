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

import { addCertification } from './actions';
import fetchCertifier from './fetch-certifier';
import checkIfCertified from './check-if-certified';

const knownCertifiers = [ 'smsverification' ];

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
      .then((certifier) => check(certifier, address))
      .then((isCertified) => {
        if (isCertified) {
          store.dispatch(addCertification(address, name));
        }
      })
      .catch((err) => {
        if (err) {
          console.error(`Failed to check if ${address} certified by ${name}:`, err);
        }
      });
    });
  };
};
