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

import sinon from 'sinon';

import Manager, { events } from './manager';

function newStub () {
  const manager = new Manager({});

  manager._eth = {
    isStarted: false,
    start: () => manager._updateSubscriptions(manager.__test, null, 'test')
  };

  manager._personal = {
    isStarted: false,
    start: () => manager._updateSubscriptions(manager.__test, null, 'test')
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
      expect(Object.keys(manager.subscriptions)).to.deep.equal(events);
      expect(Object.keys(manager.values)).to.deep.equal(events);
    });
  });

  describe('subscriptions', () => {
    events.filter((event) => event.indexOf('_') !== -1).forEach((event) => {
      const [engineName] = event.split('_');
      let engine;
      let cb;
      let subscriptionId;

      describe(event, () => {
        beforeEach(() => {
          engine = manager[`_${engineName}`];
          manager.__test = event;
          cb = sinon.stub();
          sinon.spy(engine, 'start');
          return manager
            .subscribe(event, cb)
            .then((_subscriptionId) => {
              subscriptionId = _subscriptionId;
            });
        });

        it(`puts the ${engineName} engine in a started state`, () => {
          expect(engine.start).to.have.been.called;
        });

        it('returns a subscriptionId', () => {
          expect(subscriptionId).to.be.ok;
        });

        it('calls the subscription callback with updated values', () => {
          expect(cb).to.have.been.calledWith(null, 'test');
        });
      });
    });
  });
});
