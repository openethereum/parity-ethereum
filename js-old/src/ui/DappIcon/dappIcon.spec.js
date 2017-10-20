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

import DappIcon from './';

const DAPPS_URL = 'http://test';

let api;
let component;

function createApi () {
  api = {
    dappsUrl: DAPPS_URL
  };

  return api;
}

function render (props = {}) {
  if (!props.app) {
    props.app = {};
  }

  component = shallow(
    <DappIcon { ...props } />,
    {
      context: { api: createApi() }
    }
  );

  return component;
}

describe('ui/DappIcon', () => {
  it('renders defaults', () => {
    expect(render()).to.be.ok;
  });

  it('adds specified className', () => {
    expect(render({ className: 'testClass' }).hasClass('testClass')).to.be.true;
  });

  it('renders local apps with correct URL', () => {
    expect(render({ app: { id: 'test', type: 'local', iconUrl: 'test.img' } }).props().src).to.equal(
      `${DAPPS_URL}/test/test.img`
    );
  });

  it('renders other apps with correct URL', () => {
    expect(render({ app: { id: 'test', image: '/test.img' } }).props().src).to.equal(
      `/test.img`
    );
  });
});
