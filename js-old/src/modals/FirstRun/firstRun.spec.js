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

import FirstRun from './';

let component;
let onClose;

function createApi () {
  return {};
}

function createRedux () {
  return {
    dispatch: sinon.stub(),
    subscribe: sinon.stub(),
    getState: () => {
      return {
        personal: {
          hasAccounts: false
        }
      };
    }
  };
}

function render (props = { visible: true }) {
  onClose = sinon.stub();
  component = shallow(
    <FirstRun
      { ...props }
      onClose={ onClose }
    />,
    {
      context: {
        store: createRedux()
      }
    }
  ).find('FirstRun').shallow({
    context: {
      api: createApi()
    }
  });

  return component;
}

describe('modals/FirstRun', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });
});
