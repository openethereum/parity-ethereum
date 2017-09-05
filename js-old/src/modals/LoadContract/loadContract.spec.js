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

import LoadContract from './';

const CONTRACTS = {
  345: { id: 345, name: 'test345' },
  456: { id: 456, name: 'test456' },
  987: { id: 987, name: 'test987' }
};
const SNIPPETS = {
  123: { id: 123, name: 'test123' }
};

let component;
let instance;
let onClose;
let onDelete;
let onLoad;

function render () {
  onClose = sinon.stub();
  onDelete = sinon.stub();
  onLoad = sinon.stub();
  component = shallow(
    <LoadContract
      contracts={ CONTRACTS }
      onClose={ onClose }
      onDelete={ onDelete }
      onLoad={ onLoad }
      snippets={ SNIPPETS }
    />
  );
  instance = component.instance();

  return component;
}

describe('modals/LoadContract', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('event methods', () => {
    describe('handleChangeTab', () => {
      beforeEach(() => {
        instance.onClickContract(null, 345);
        instance.handleChangeTab();
      });

      it('resets the selected value', () => {
        expect(instance.state.selected).to.equal(-1);
      });
    });

    describe('onClickContract', () => {
      beforeEach(() => {
        instance.onClickContract(null, 456);
      });

      it('sets the selected value', () => {
        expect(instance.state.selected).to.equal(456);
      });
    });

    describe('onClose', () => {
      beforeEach(() => {
        instance.onClose();
      });

      it('calls onClose', () => {
        expect(onClose).to.have.been.called;
      });
    });

    describe('onLoad', () => {
      beforeEach(() => {
        instance.onLoad();
      });

      it('calls onLoad', () => {
        expect(onLoad).to.have.been.called;
      });

      it('calls onClose', () => {
        expect(onClose).to.have.been.called;
      });
    });

    describe('onDeleteRequest', () => {
      beforeEach(() => {
        instance.onDeleteRequest(987);
      });

      it('sets deleteRequest true', () => {
        expect(instance.state.deleteRequest).to.be.true;
      });

      it('sets the deleteId', () => {
        expect(instance.state.deleteId).to.equal(987);
      });
    });

    describe('onConfirmRemoval', () => {
      beforeEach(() => {
        instance.onDeleteRequest(987);
        instance.onConfirmRemoval();
      });

      it('calls onDelete', () => {
        expect(onDelete).to.have.been.calledWith(987);
      });

      it('sets deleteRequest false', () => {
        expect(instance.state.deleteRequest).to.be.false;
      });

      it('clears the deleteId', () => {
        expect(instance.state.deleteId).to.equal(-1);
      });
    });

    describe('onRejectRemoval', () => {
      beforeEach(() => {
        instance.onDeleteRequest(987);
        instance.onRejectRemoval();
      });

      it('sets deleteRequest false', () => {
        expect(instance.state.deleteRequest).to.be.false;
      });

      it('clears the deleteId', () => {
        expect(instance.state.deleteId).to.equal(-1);
      });
    });
  });
});
