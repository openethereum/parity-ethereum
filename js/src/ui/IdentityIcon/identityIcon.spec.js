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

import IdentityIcon from './';

const ADDRESS0 = '0x0000000000000000000000000000000000000000';
const ADDRESS1 = '0x0123456789012345678901234567890123456789';
const ADDRESS2 = '0x9876543210987654321098765432109876543210';

let component;
let instance;

function createApi () {
  return {
    dappsUrl: 'dappsUrl/'
  };
}

function createRedux () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        images: {
          [ADDRESS2]: 'reduxImage'
        }
      };
    }
  };
}

function render (props = {}) {
  if (props && props.address === undefined) {
    props.address = ADDRESS1;
  }

  component = shallow(
    <IdentityIcon { ...props } />,
    { context: { store: createRedux() } }
  ).find('IdentityIcon').shallow({ context: { api: createApi() } });

  instance = component.instance();
  instance.componentDidMount();

  return component;
}

describe('ui/IdentityIcon', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  describe('images', () => {
    it('renders an <img> with address specified', () => {
      const img = render().find('img');

      expect(img).to.have.length(1);
      expect(img.props().src).to.equal('test-createIdentityImg');
    });

    it('renders an <img> with redux source when available', () => {
      const img = render({ address: ADDRESS2 }).find('img');

      expect(img).to.have.length(1);
      expect(img.props().src).to.equal('dappsUrl/reduxImage');
    });

    it('renders an <ContractIcon> with no address specified', () => {
      expect(render({ address: null }).find('ActionCode')).to.have.length(1);
    });

    it('renders an <CancelIcon> with 0x00..00 address specified', () => {
      expect(render({ address: ADDRESS0 }).find('ContentClear')).to.have.length(1);
    });
  });

  describe('sizes', () => {
    it('renders 56px by default', () => {
      expect(render().find('img').props().width).to.equal('56px');
    });

    it('renders 16px for tiny', () => {
      expect(render({ tiny: true }).find('img').props().width).to.equal('16px');
    });

    it('renders 24px for button', () => {
      expect(render({ button: true }).find('img').props().width).to.equal('24px');
    });

    it('renders 32px for inline', () => {
      expect(render({ inline: true }).find('img').props().width).to.equal('32px');
    });
  });
});
