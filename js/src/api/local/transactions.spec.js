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

import transactions from './transactions';
import { TransportError } from '../transport/error';

const DUMMY_TX = 'dummy';

describe('api/local/transactions', () => {
  beforeEach(() => {
    transactions.reset();
  });

  it('can store transactions', () => {
    const id1 = transactions.add(DUMMY_TX);
    const id2 = transactions.add(DUMMY_TX);
    const requests = transactions.requestsToConfirm();

    expect(id1).to.be.equal('0x1');
    expect(id2).to.be.equal('0x2');
    expect(requests.length).to.be.equal(2);
    expect(requests[0].id).to.be.equal(id1);
    expect(requests[1].id).to.be.equal(id2);
    expect(requests[0].payload.sendTransaction).to.be.equal(DUMMY_TX);
    expect(requests[1].payload.sendTransaction).to.be.equal(DUMMY_TX);
  });

  it('can confirm transactions', () => {
    const id1 = transactions.add(DUMMY_TX);
    const id2 = transactions.add(DUMMY_TX);

    const hash1 = '0x1111111111111111111111111111111111111111';
    const hash2 = '0x2222222222222222222222222222222222222222';

    transactions.confirm(id1, hash1);
    transactions.confirm(id2, hash2);

    const requests = transactions.requestsToConfirm();

    expect(requests.length).to.be.equal(0);
    expect(transactions.hash(id1)).to.be.equal(hash1);
    expect(transactions.hash(id2)).to.be.equal(hash2);
  });

  it('can reject transactions', () => {
    const id = transactions.add(DUMMY_TX);

    transactions.reject(id);

    const requests = transactions.requestsToConfirm();

    expect(requests.length).to.be.equal(0);
    expect(() => transactions.hash(id)).to.throw(TransportError);
  });

  it('can lock and confirm transactions', () => {
    const id = transactions.add(DUMMY_TX);
    const hash = '0x1111111111111111111111111111111111111111';

    transactions.lock(id);

    const requests = transactions.requestsToConfirm();

    expect(requests.length).to.be.equal(0);
    expect(transactions.get(id)).to.be.null;
    expect(transactions.hash(id)).to.be.null;

    transactions.confirm(id, hash);

    expect(transactions.hash(id)).to.be.equal(hash);
  });
});
