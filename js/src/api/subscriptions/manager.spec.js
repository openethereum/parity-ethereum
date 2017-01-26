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

import sinon from 'sinon';

import Manager, { events } from './manager';

function newStub () {
  const start = () => manager._updateSubscriptions(manager.__test, null, 'test');

  const manager = new Manager({
    transport: {
      isConnected: true
    }
  });

  manager._eth = {
    isStarted: false,
    start
  };

  manager._personal = {
    isStarted: false,
    start
  };

  manager._signer = {
    isStarted: false,
    start
  };

  return manager;
}

describe('api/subscriptions/manager', () => {
  let manager;

  beforeEach(() => {
    manager = newStub();
  });

  describe('constructor', () => {
    it('sets up the subscription types & defaults', () => {
      expect(manager.subscriptions).to.be.an.array;
      expect(Object.keys(manager.values)).to.deep.equal(Object.keys(events));
    });
  });

  describe('subscriptions', () => {
    Object
      .keys(events)
      .filter((eventName) => eventName.indexOf('_') !== -1)
      .forEach((eventName) => {
        const { module } = events[eventName];
        let engine;
        let cb;
        let subscriptionId;

        describe(eventName, () => {
          beforeEach(() => {
            engine = manager[`_${module}`];
            manager.__test = eventName;
            cb = sinon.stub();
            sinon.spy(engine, 'start');

            return manager
              .subscribe(eventName, cb)
              .then((_subscriptionId) => {
                subscriptionId = _subscriptionId;
              });
          });

          it(`puts the ${module} engine in a started state`, () => {
            expect(engine.start).to.have.been.called;
          });

          it('returns a subscriptionId', () => {
            expect(subscriptionId).to.be.a.number;
          });

          it('calls the subscription callback with updated values', () => {
            expect(cb).to.have.been.calledWith(null, 'test');
          });
        });
      });
  });

  describe('unsubscriptions', () => {
    Object
      .keys(events)
      .filter((eventName) => eventName.indexOf('_') !== -1)
      .forEach((eventName) => {
        const { module } = events[eventName];
        let engine;
        let cb;

        describe(eventName, () => {
          beforeEach(() => {
            engine = manager[`_${module}`];
            manager.__test = eventName;
            cb = sinon.stub();
            sinon.spy(engine, 'start');

            return manager
              .subscribe(eventName, cb)
              .then((_subscriptionId) => {
                manager.unsubscribe(_subscriptionId);
              })
              .then(() => {
                manager._updateSubscriptions(manager.__test, null, 'test2');
              });
          });

          it('does not call the callback after unsibscription', () => {
            expect(cb).to.have.been.calledWith(null, 'test');
            expect(cb).to.not.have.been.calledWith(null, 'test2');
          });
        });
      });
  });
});
