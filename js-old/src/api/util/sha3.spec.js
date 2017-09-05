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

import { sha3 } from './sha3';

describe('api/util/sha3', () => {
  describe('sha3', () => {
    it('constructs a correct sha3 value', () => {
      expect(sha3('jacogr')).to.equal('0x2f4ff4b5a87abbd2edfed699db48a97744e028c7f7ce36444d40d29d792aa4dc');
    });

    it('constructs a correct sha3 encoded as hex', () => {
      const key = '000000000000000000000000391694e7e0b0cce554cb130d723a9d27458f9298' + '0000000000000000000000000000000000000000000000000000000000000001';

      expect(sha3(key, { encoding: 'hex' })).to.equal('0x6661e9d6d8b923d5bbaab1b96e1dd51ff6ea2a93520fdc9eb75d059238b8c5e9');
      expect(sha3(`0x${key}`, { encoding: 'hex' })).to.equal('0x6661e9d6d8b923d5bbaab1b96e1dd51ff6ea2a93520fdc9eb75d059238b8c5e9');
    });

    it('constructs a correct sha3 from Uint8Array', () => {
      expect(sha3('01020304', { encoding: 'hex' })).to.equal('0xa6885b3731702da62e8e4a8f584ac46a7f6822f4e2ba50fba902f67b1588d23b');
      expect(sha3(Uint8Array.from([1, 2, 3, 4]))).to.equal('0xa6885b3731702da62e8e4a8f584ac46a7f6822f4e2ba50fba902f67b1588d23b');
    });

    it('should interpret as bytes by default', () => {
      expect(sha3('0x01020304')).to.equal('0xa6885b3731702da62e8e4a8f584ac46a7f6822f4e2ba50fba902f67b1588d23b');
    });

    it('should force text if option is passed', () => {
      expect(sha3('0x01020304', { encoding: 'raw' })).to.equal('0x16bff43de576d28857dcba65a56fc17c5e93c09bd6a709268eff8e62025ae869');
      expect(sha3.text('0x01020304')).to.equal('0x16bff43de576d28857dcba65a56fc17c5e93c09bd6a709268eff8e62025ae869');
    });
  });
});
