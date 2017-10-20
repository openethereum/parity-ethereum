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

import React from 'react';
import sinon from 'sinon';
import { shallow } from 'enzyme';

import '../../../../environment/tests';

import CallsToolbar from './CallsToolbar';

describe('views/Status/components/CallsToolbar', () => {
  const callEl = { offsetTop: 0 };
  const containerEl = { scrollTop: 0, clientHeight: 0, scrollHeight: 999 };

  describe('rendering (no call)', () => {
    let rendered;

    before(() => {
      const call = null;

      rendered = shallow(<CallsToolbar call={ call } callEl={ callEl } containerEl={ containerEl } />);
    });

    it('does not render the component', () => {
      expect(rendered).to.not.have.descendants('[data-test="CallsToolbar-button-more"]');
    });
  });

  describe('rendering', () => {
    const call = { callNo: 456, name: 'eth_call', params: '', response: '' };
    let rendered;
    let btncontainer;

    before(() => {
      rendered = shallow(<CallsToolbar call={ call } callEl={ callEl } containerEl={ containerEl } />);
      btncontainer = rendered.find('[data-test="CallsToolbar-button-container"]');
    });

    it('renders the More button', () => {
      expect(rendered).to.have.descendants('[data-test="CallsToolbar-button-more"]');
    });

    it('renders the Set button', () => {
      expect(btncontainer).to.have.descendants('[data-test="CallsToolbar-button-setCall"]');
    });

    it('renders the Fire button', () => {
      expect(btncontainer).to.have.descendants('[data-test="CallsToolbar-button-makeCall"]');
    });

    it('renders the Copy button', () => {
      expect(btncontainer).to.have.descendants('[data-test="CallsToolbar-copyCallToClipboard"]');
    });
  });

  describe('actions', () => {
    const call = { callNo: 456, name: 'eth_call', params: '', response: '' };
    const actions = { fireRpc: sinon.stub(), copyToClipboard: sinon.stub(), selectRpcMethod: sinon.stub() };

    let rendered;
    let instance;

    before(() => {
      rendered = shallow(<CallsToolbar call={ call } callEl={ callEl } containerEl={ containerEl } actions={ actions } />);
      instance = rendered.instance();
    });

    it('calls copyToClipboard with action copyToClipboard', () => {
      instance.copyToClipboard();
      expect(actions.copyToClipboard).to.be.calledOnce;
    });

    it('calls setCall with action selectRpcMethod', () => {
      instance.setCall();
      expect(actions.selectRpcMethod).to.be.calledOnce;
    });

    it('calls makeCall with action fireRpc', () => {
      instance.makeCall();
      expect(actions.fireRpc).to.be.calledOnce;
    });
  });

  describe('utility', () => {
    const call = { callNo: 456, name: 'eth_call', params: '', response: '' };
    let rendered;
    let instance;

    before(() => {
      rendered = shallow(<CallsToolbar call={ call } callEl={ callEl } containerEl={ containerEl } />);
      instance = rendered.instance();
    });

    describe('.hasScrollbar', () => {
      it('correctly returns true when scrollbar', () => {
        expect(instance.hasScrollbar({ clientHeight: 123, scrollHeight: 456 })).to.be.true;
      });

      it('correctly returns false when no scrollbar', () => {
        expect(instance.hasScrollbar({ clientHeight: 456, scrollHeight: 123 })).to.be.false;
      });
    });
  });
});
