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

import Contracts from '../../contracts';

export function setCode (address, code) {
  return {
    type: 'setCode',
    address, code
  };
}

export function setMethod (signature, method) {
  return {
    type: 'methodSignatureLookup',
    signature, method
  };
}

export function fetchCode (address) {
  return (dispatch, getState) => {
    let { api } = getState();

    api.eth
      .getCode(address)
      .then(code => {
        dispatch(setCode(address, code));
      })
      .catch(e => {
        console.error('methodDecoder::fetchCode', e);
      });
  };
}

export function fetchMethod (signature) {
  return (dispatch, getState) => {
    Contracts
      .get()
      .signatureReg.lookup(signature)
      .then(method => {
        dispatch(setMethod(signature, method));
      })
      .catch(e => {
        console.error('methodDecoder::fetchMethod', e);
      });
  };
}
