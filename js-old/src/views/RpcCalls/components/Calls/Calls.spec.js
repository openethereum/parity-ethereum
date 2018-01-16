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
import { shallow } from 'enzyme';

import '../../../../environment/tests';

import Calls from './Calls';

describe('views/Status/components/Calls', () => {
  describe('rendering (no calls)', () => {
    let rendered;

    before(() => {
      const calls = [];

      rendered = shallow(<Calls calls={ calls } />);
    });

    it('renders the component and container', () => {
      expect(rendered).to.be.ok;
      expect(rendered).to.have.className('calls-container');
    });

    it('renders no calls', () => {
      expect(rendered.find('div[data-test="Calls-empty-wrapper"]')).to.have.exactly(1).descendants('h3');
    });

    it('renders no clear button', () => {
      expect(rendered.find('a[data-test="Calls-remove"]')).to.not.exist;
    });

    it('renders an attached CallsToolbar', () => {
      expect(rendered).to.have.exactly(1).descendants('CallsToolbar');
    });
  });

  describe('rendering (calls supplied)', () => {
    const calls = [
      { callNo: 0, name: 'eth_call', params: '', response: '' },
      { callNo: 1, name: 'eth_sendTransaction', params: '', response: '' }
    ];
    const actions = { action1: true, action2: true };

    let rendered;
    let instance;

    before(() => {
      rendered = shallow(<Calls calls={ calls } actions={ actions } />);
      instance = rendered.instance();
    });

    it('renders the clear button', () => {
      expect(rendered).to.have.exactly(1).descendants('a[data-test="Calls-remove"]');
    });

    it('renders calls', () => {
      expect(rendered.find('div[data-test="Calls-empty-wrapper"]')).to.not.exist;
      expect(rendered.find('div.row div')).to.have.exactly(2).descendants('Call');
    });

    it('passes the correct properties to Call', () => {
      const call = rendered.find('Call').first();

      expect(call).to.have.prop('setActiveCall', instance.setActiveCall);
      expect(call).to.have.prop('call').deep.equal(calls[0]);
    });

    it('passes the correct properties to CallsToolbar', () => {
      const child = { offsetTop: 0 };
      const container = { scrollTop: 0 };

      instance.setCallsHistory(container);
      rendered.setState({ activeCall: 'dummyActiveCall', activeChild: child });

      const toolbar = rendered.find('CallsToolbar').first();

      expect(toolbar).to.have.prop('call', 'dummyActiveCall');
      expect(toolbar).to.have.prop('actions').deep.equal(actions);
      expect(toolbar).to.have.prop('callEl').deep.equal(child);
      expect(toolbar).to.have.prop('containerEl').deep.equal(container);
    });
  });

  describe('actions', () => {
    let rendered;
    let instance;

    before(() => {
      const calls = [
        { callNo: 0, name: 'eth_call', params: '', response: '' },
        { callNo: 1, name: 'eth_sendTransaction', params: '', response: '' }
      ];

      rendered = shallow(<Calls calls={ calls } />);
      instance = rendered.instance();
    });

    it('sets the element via setCallsHistory', () => {
      instance.setCallsHistory('dummyElement');

      expect(instance._callsHistory).to.equal('dummyElement');
    });

    it('sets state via setActiveCall', () => {
      instance.setActiveCall('dummyActiveCall', 'dummyActiveChild');

      expect(rendered).to.have.state('activeCall', 'dummyActiveCall');
      expect(rendered).to.have.state('activeChild', 'dummyActiveChild');
    });

    it('clears state via clearActiveCall', () => {
      instance.setActiveCall('dummyActiveCall', 'dummyActiveChild');
      expect(rendered).to.have.state('activeCall', 'dummyActiveCall');
      instance.clearActiveCall();

      expect(rendered).to.have.state('activeCall', null);
      expect(rendered).to.have.state('activeChild', null);
    });
  });
});
