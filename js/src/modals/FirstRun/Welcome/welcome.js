// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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

import React, { Component } from 'react';

import imagesEthcore from '../../../../assets/images/parity-logo-white.svg';

const LOGO_STYLE = {
  float: 'right',
  width: '25%',
  height: 'auto',
  margin: '0 1.5em'
};

export default class FirstRun extends Component {
  render () {
    return (
      <div>
        <img
          src={ imagesEthcore }
          alt='Ethcore Ltd.'
          style={ LOGO_STYLE }
        />
        <p>Welcome to <strong>Parity</strong>, the fastest and simplest way to run your node.</p>
        <p>The next few steps will guide you through the process of setting up you Parity instance and the associated account.</p>
        <p>Click <strong>Next</strong> to continue your journey.</p>
      </div>
    );
  }
}
