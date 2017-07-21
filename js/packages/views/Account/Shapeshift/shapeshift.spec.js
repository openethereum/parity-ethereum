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

import { shallow } from 'enzyme';
import React from 'react';
import sinon from 'sinon';

import { STAGE_COMPLETED, STAGE_OPTIONS, STAGE_WAIT_DEPOSIT, STAGE_WAIT_EXCHANGE } from './store';
import Shapeshift from './';

const ADDRESS = '0x0123456789012345678901234567890123456789';

let component;
let instance;
let onClose;

function render (props = {}) {
  onClose = sinon.stub();
  component = shallow(
    <Shapeshift
      address={ ADDRESS }
      onClose={ onClose }
      { ...props }
    />,
    { context: { store: {} } }
  );
  instance = component.instance();

  return component;
}

describe('views/Account/Shapeshift', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('componentDidMount', () => {
    beforeEach(() => {
      render();
      sinon.stub(instance.store, 'retrieveCoins');
      return instance.componentDidMount();
    });

    afterEach(() => {
      instance.store.retrieveCoins.restore();
    });

    it('retrieves the list of coins when mounting', () => {
      expect(instance.store.retrieveCoins).to.have.been.called;
    });
  });

  describe('componentWillUnmount', () => {
    beforeEach(() => {
      render();
      sinon.stub(instance.store, 'unsubscribe');
      return instance.componentWillUnmount();
    });

    afterEach(() => {
      instance.store.unsubscribe.restore();
    });

    it('removes any subscriptions when unmounting', () => {
      expect(instance.store.unsubscribe).to.have.been.called;
    });
  });

  describe('renderDialogActions', () => {
    beforeEach(() => {
      render();
    });

    describe('shift button', () => {
      beforeEach(() => {
        sinon.stub(instance.store, 'shift').resolves();

        instance.store.setCoins(['BTC']);
        instance.store.toggleAcceptTerms();
      });

      afterEach(() => {
        instance.store.shift.restore();
      });

      it('disabled shift button when not accepted', () => {
        instance.store.toggleAcceptTerms();
        expect(shallow(instance.renderDialogActions()[2]).props().disabled).to.be.true;
      });

      it('shows shift button when accepted', () => {
        expect(shallow(instance.renderDialogActions()[2]).props().disabled).to.be.false;
      });

      it('calls the shift on store when clicked', () => {
        shallow(instance.renderDialogActions()[2]).simulate('touchTap');
        expect(instance.store.shift).to.have.been.called;
      });
    });
  });

  describe('renderPage', () => {
    beforeEach(() => {
      render();
    });

    it('renders ErrorStep on error, passing the store', () => {
      instance.store.setError('testError');
      const page = instance.renderPage();

      expect(page.type).to.match(/ErrorStep/);
      expect(page.props.store).to.equal(instance.store);
    });

    it('renders OptionsStep with STAGE_OPTIONS, passing the store', () => {
      instance.store.setStage(STAGE_OPTIONS);
      const page = instance.renderPage();

      expect(page.type).to.match(/OptionsStep/);
      expect(page.props.store).to.equal(instance.store);
    });

    it('renders AwaitingDepositStep with STAGE_WAIT_DEPOSIT, passing the store', () => {
      instance.store.setStage(STAGE_WAIT_DEPOSIT);
      const page = instance.renderPage();

      expect(page.type).to.match(/AwaitingDepositStep/);
      expect(page.props.store).to.equal(instance.store);
    });

    it('renders AwaitingExchangeStep with STAGE_WAIT_EXCHANGE, passing the store', () => {
      instance.store.setStage(STAGE_WAIT_EXCHANGE);
      const page = instance.renderPage();

      expect(page.type).to.match(/AwaitingExchangeStep/);
      expect(page.props.store).to.equal(instance.store);
    });

    it('renders CompletedStep with STAGE_COMPLETED, passing the store', () => {
      instance.store.setStage(STAGE_COMPLETED);
      const page = instance.renderPage();

      expect(page.type).to.match(/CompletedStep/);
      expect(page.props.store).to.equal(instance.store);
    });
  });
});
