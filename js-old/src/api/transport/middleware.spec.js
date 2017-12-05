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

import Middleware from './middleware';
import JsonRpcBase from './jsonRpcBase';

const MOCKED = 'mocked!';

class MockTransport extends JsonRpcBase {
  _execute () {
    return Promise.resolve(MOCKED);
  }
}

class MockMiddleware extends Middleware {
  constructor (transport) {
    super(transport);

    this.register('mock_rpc', ([num]) => num);
    this.register('mock_null', () => null);
  }
}

describe('api/transport/Middleware', () => {
  let transport;

  beforeEach(() => {
    transport = new MockTransport();
    transport.addMiddleware(MockMiddleware);
  });

  it('Routes requests to middleware', () => {
    return transport.execute('mock_rpc', 100).then((num) => {
      expect(num).to.be.equal(100);
    });
  });

  it('Passes non-mocked requests through', () => {
    return transport.execute('not_moced', 200).then((result) => {
      expect(result).to.be.equal(MOCKED);
    });
  });

  it('Passes mocked requests through, if middleware returns null', () => {
    return transport.execute('mock_null', 300).then((result) => {
      expect(result).to.be.equal(MOCKED);
    });
  });
});
