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

import QrScan from './';

let component;
let onError;
let onScan;

function render () {
  onError = sinon.stub();
  onScan = sinon.stub();

  component = shallow(
    <QrScan
      onError={ onError }
      onScan={ onScan }
    />
  );

  return component;
}

describe('ui/QrScan', () => {
  beforeEach(() => {
    render();
  });

  it('renders defaults', () => {
    expect(component).to.be.ok;
  });

  describe('Reader', () => {
    let reader;

    beforeEach(() => {
      reader = component.find('Reader');
    });

    it('renders component', () => {
      expect(reader.get(0)).to.be.ok;
    });

    it('attaches onError', () => {
      expect(reader.props().onError).to.equal(onError);
    });

    it('attaches onScan', () => {
      expect(reader.props().onScan).to.equal(onScan);
    });
  });
});
