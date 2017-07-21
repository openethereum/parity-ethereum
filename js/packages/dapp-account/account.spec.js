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

import { ACCOUNTS, ADDRESS, createRedux } from './account.test.js';

import Account from './account';

let component;
let instance;
let store;

function render (props) {
  component = shallow(
    <Account
      accounts={ ACCOUNTS }
      params={ { address: ADDRESS } }
      { ...props }
    />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('Account').shallow();
  instance = component.instance();
  store = instance.store;

  return component;
}

describe('views/Account', () => {
  describe('rendering', () => {
    beforeEach(() => {
      render();
    });

    it('renders defaults', () => {
      expect(component).to.be.ok;
    });

    describe('sections', () => {
      it('renders the Actionbar', () => {
        expect(component.find('Actionbar')).to.have.length(1);
      });

      it('renders the Page', () => {
        expect(component.find('Page')).to.have.length(1);
      });

      it('renders the Header', () => {
        expect(component.find('Header')).to.have.length(1);
      });

      it('renders the Transactions', () => {
        expect(component.find('Connect(Transactions)')).to.have.length(1);
      });

      it('renders no other sections', () => {
        expect(component.find('div').children()).to.have.length(2);
      });
    });
  });

  describe('sub-renderers', () => {
    describe('renderActionBar', () => {
      let bar;

      beforeEach(() => {
        render();

        bar = instance.renderActionbar({ tokens: {} });
      });

      it('renders the bar', () => {
        expect(bar.type).to.match(/Actionbar/);
      });
    });

    describe('renderDeleteDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isDeleteVisible).to.be.false;
        expect(instance.renderDeleteDialog(ACCOUNTS[ADDRESS])).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.toggleDeleteDialog();
        expect(instance.renderDeleteDialog(ACCOUNTS[ADDRESS]).type).to.match(/Connect/);
      });
    });

    describe('renderEditDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isEditVisible).to.be.false;
        expect(instance.renderEditDialog(ACCOUNTS[ADDRESS])).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.toggleEditDialog();
        expect(instance.renderEditDialog(ACCOUNTS[ADDRESS]).type).to.match(/Connect/);
      });
    });

    describe('renderFundDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isFundVisible).to.be.false;
        expect(instance.renderFundDialog()).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.toggleFundDialog();
        expect(instance.renderFundDialog().type).to.match(/Shapeshift/);
      });
    });

    describe('renderPasswordDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isPasswordVisible).to.be.false;
        expect(instance.renderPasswordDialog()).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.togglePasswordDialog();
        expect(instance.renderPasswordDialog({ address: ADDRESS }).type).to.match(/Connect/);
      });
    });

    describe('renderTransferDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isTransferVisible).to.be.false;
        expect(instance.renderTransferDialog()).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.toggleTransferDialog();
        expect(instance.renderTransferDialog().type).to.match(/Connect/);
      });
    });

    describe('renderVerificationDialog', () => {
      it('renders null when not visible', () => {
        render();

        expect(store.isVerificationVisible).to.be.false;
        expect(instance.renderVerificationDialog()).to.be.null;
      });

      it('renders the modal when visible', () => {
        render();

        store.toggleVerificationDialog();
        expect(instance.renderVerificationDialog().type).to.match(/Connect/);
      });
    });
  });
});
