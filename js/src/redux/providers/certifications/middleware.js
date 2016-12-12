// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import Contracts from '~/contracts';
import { addCertification } from './actions';

const knownCertifiers = [ 'smsverification' ];

export default class CertificationsMiddleware {
  toMiddleware () {
    return (store) => (next) => (action) => {
      if (action.type !== 'fetchCertifications') {
        return next(action);
      }

      const { address } = action;
      const badgeReg = Contracts.get().badgeReg;

      knownCertifiers.forEach((name) => {
        badgeReg.fetchCertifier(name)
          .then((cert) => {
            return badgeReg.checkIfCertified(cert.address, address)
              .then((isCertified) => {
                if (isCertified) {
                  const { name, title, icon } = cert;
                  store.dispatch(addCertification(address, name, title, icon));
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
  }
}
