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

import { spy, stub, useFakeTimers } from 'sinon';

import subscribeToEvents from './subscribe-to-events';
import {
  pastLogs, liveLogs, createApi, createContract
} from './subscribe-to-events.test.js';

// Note: We want to have a `setTimeout` that is independent from
// `sinon.useFakeTimers`. Therefore we dereference `setTimeout` here.
const _setTimeout = setTimeout;
const delay = (t) => new Promise((resolve) => {
  _setTimeout(resolve, t);
});

describe('util/subscribe-to-events', () => {
  beforeEach(function () {
    this.api = createApi();
    this.contract = createContract(this.api);
  });

  it('installs a filter', async function () {
    const { api, contract } = this;

    subscribeToEvents(contract, [ 'Foo', 'Bar' ]);
    await delay(0);

    expect(api.eth.newFilter.calledOnce).to.equal(true);
    expect(api.eth.newFilter.firstCall.args).to.eql([ {
      fromBlock: 0, toBlock: 'latest',
      address: contract.address,
      topics: [ [
        contract.instance.Foo.signature,
        contract.instance.Bar.signature
      ] ]
    } ]);
  });

  it('queries & parses logs in the beginning', async function () {
    const { api, contract } = this;

    subscribeToEvents(contract, [ 'Foo', 'Bar' ]);

    await delay(0);
    expect(api.eth.getFilterLogs.callCount).to.equal(1);
    expect(api.eth.getFilterLogs.firstCall.args).to.eql([ 123 ]);

    await delay(0);
    expect(contract.parseEventLogs.callCount).to.equal(1);
  });

  it('emits logs in the beginning', async function () {
    const { contract } = this;

    const onLog = spy();
    const onFoo = spy();
    const onBar = spy();

    subscribeToEvents(contract, [ 'Foo', 'Bar' ])
      .on('log', onLog)
      .on('Foo', onFoo)
      .on('Bar', onBar);

    await delay(0);

    expect(onLog.callCount).to.equal(2);
    expect(onLog.firstCall.args).to.eql([ pastLogs[0] ]);
    expect(onLog.secondCall.args).to.eql([ pastLogs[1] ]);
    expect(onFoo.callCount).to.equal(1);
    expect(onFoo.firstCall.args).to.eql([ pastLogs[0] ]);
    expect(onBar.callCount).to.equal(1);
    expect(onBar.firstCall.args).to.eql([ pastLogs[1] ]);
  });

  it('uninstalls the filter on sunsubscribe', async function () {
    const { api, contract } = this;

    const s = subscribeToEvents(contract, [ 'Foo', 'Bar' ]);

    await delay(0);
    s.unsubscribe();
    await delay(0);

    expect(api.eth.uninstallFilter.calledOnce).to.equal(true);
    expect(api.eth.uninstallFilter.firstCall.args).to.eql([ 123 ]);
  });

  it('checks for new events regularly', async function () {
    const clock = useFakeTimers();
    const { api, contract } = this;

    api.eth.getFilterLogs = stub().resolves([]);

    const onLog = spy();
    const onBar = spy();

    subscribeToEvents(contract, [ 'Bar' ], { interval: 5 })
      .on('log', onLog)
      .on('Bar', onBar);
    await delay(1); // let stubs resolve
    clock.tick(5);
    await delay(1); // let stubs resolve

    expect(onLog.callCount).to.be.at.least(1);
    expect(onLog.firstCall.args).to.eql([ liveLogs[0] ]);
    expect(onBar.callCount).to.be.at.least(1);
    expect(onBar.firstCall.args).to.eql([ liveLogs[0] ]);

    clock.restore();
  });

  it('accepts a custom block range', async function () {
    const { api, contract } = this;

    subscribeToEvents(contract, [ 'Foo' ], { from: 123, to: 321 });

    await delay(0);
    expect(api.eth.newFilter.callCount).to.equal(1);
    expect(api.eth.newFilter.firstCall.args[0].fromBlock).to.equal(123);
    expect(api.eth.newFilter.firstCall.args[0].toBlock).to.equal(321);
  });
});
