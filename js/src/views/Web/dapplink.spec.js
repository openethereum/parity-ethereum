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

import { BASE_URL, decode, encodePath, encodeUrl } from './dapplink';

const TEST_TOKEN = 'token';
const TEST_URL = 'https://parity.io';
const TEST_URL_LONG = 'http://some.very.very.very.long.long.long.domain.example.com';
const TEST_PREFIX = 'EHQPPSBE5DM78X3GECX2YBVGC5S6JX3S5SMPY';
const TEST_PREFIX_LONG = [
  'EHQPPSBE5DM78X3G78QJYWVFDNJJWXK5E9WJWXK5E9WJWXK5E9WJWV3FDSKJWV3', 'FDSKJWV3FDSKJWS3FDNGPJVHECNW62VBGDHJJWRVFDM'
].join('.');
const TEST_RESULT = `${TEST_PREFIX}${BASE_URL}`;
const TEST_ENCODED = `${TEST_TOKEN}+${TEST_URL}`;

describe('util/ethlink', () => {
  describe('decode', () => {
    it('decodes into encoded url', () => {
      expect(decode(TEST_PREFIX)).to.equal(TEST_ENCODED);
    });

    it('decodes full into encoded url', () => {
      expect(decode(TEST_RESULT)).to.equal(TEST_ENCODED);
    });
  });

  describe('encodePath', () => {
    it('encodes a url/token combination', () => {
      expect(encodePath(TEST_TOKEN, TEST_URL)).to.equal(TEST_PREFIX);
    });

    it('changes when token changes', () => {
      expect(encodePath('test-token-2', TEST_URL)).not.to.equal(TEST_PREFIX);
    });

    it('changes when url changes', () => {
      expect(encodePath(TEST_TOKEN, 'http://other.example.com')).not.to.equal(TEST_PREFIX);
    });
  });

  describe('encodeUrl', () => {
    it('encodes a url/token combination', () => {
      expect(encodeUrl(TEST_TOKEN, TEST_URL)).to.equal(TEST_RESULT);
    });

    it('changes when token changes', () => {
      expect(encodeUrl('test-token-2', TEST_URL)).not.to.equal(TEST_RESULT);
    });

    it('changes when url changes', () => {
      expect(encodeUrl(TEST_TOKEN, 'http://other.example.com')).not.to.equal(TEST_RESULT);
    });

    describe('splitting', () => {
      let encoded;

      beforeEach(() => {
        encoded = encodeUrl(TEST_TOKEN, TEST_URL_LONG);
      });

      it('splits long values into boundary parts', () => {
        expect(encoded).to.equal(`${TEST_PREFIX_LONG}${BASE_URL}`);
      });

      it('first part 63 characters', () => {
        expect(encoded.split('.')[0].length).to.equal(63);
      });
    });
  });
});
