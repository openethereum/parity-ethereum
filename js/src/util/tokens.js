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

import { range, uniq } from 'lodash';

import { hashToImageUrl } from '~/redux/util';

export function fetchTokenIds (tokenregInstance) {
  return tokenregInstance.tokenCount
    .call()
    .then((numTokens) => {
      const tokenIds = range(numTokens.toNumber());

      return tokenIds;
    });
}

export function fetchTokensInfo (api, tokenregInstace, tokenIds = []) {
  const uniqTokenIds = uniq(tokenIds);

  return Promise
    .all(uniqTokenIds.map((id) => fetchTokenInfo(api, tokenregInstace, id)));
}

export function fetchTokenInfo (api, tokenregInstace, tokenId) {
  return Promise
    .all([
      tokenregInstace.token.call({}, [tokenId]),
      tokenregInstace.meta.call({}, [tokenId, 'IMG'])
    ])
    .then(([ tokenData, image ]) => {
      const [ address, tag, format, name ] = tokenData;

      const token = {
        format: format.toString(),
        id: tokenId,
        image: hashToImageUrl(image),
        address,
        name,
        tag
      };

      return token;
    });
}
