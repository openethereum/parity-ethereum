// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import React, { Component, PropTypes } from 'react';

const styleOverlay = {
  position: 'fixed',
  top: 0,
  right: 0,
  bottom: 0,
  left: 0,
  background: 'rgba(255, 255, 255, 0.75)',
  zIndex: 20000
};

const styleModal = {
  position: 'fixed',
  top: 0,
  right: 0,
  bottom: 0,
  left: 0,
  zIndex: 20001
};

const styleBody = {
  margin: '0 auto',
  padding: '3em 2em',
  textAlign: 'center',
  maxWidth: '40em',
  background: 'rgba(25, 25, 25, 0.75)',
  color: 'rgb(208, 208, 208)',
  boxShadow: 'rgba(0, 0, 0, 0.25) 0px 14px 45px, rgba(0, 0, 0, 0.22) 0px 10px 18px'
};

const styleHeader = {
  fontSize: '1.25em',
  margin: '0 0 0.5em 0'
};

const styleContent = {
};

export default class Connection extends Component {
  static propTypes = {
    isApiConnected: PropTypes.bool,
    isPingConnected: PropTypes.bool
  }

  render () {
    const { isApiConnected, isPingConnected } = this.props;
    const isConnected = isApiConnected && isPingConnected;

    if (isConnected) {
      return null;
    }

    return (
      <div>
        <div style={ styleOverlay } />
        <div style={ styleModal }>
          <div style={ styleBody }>
            <div style={ styleHeader }>
              Connecting to Parity
            </div>
            <div style={ styleContent }>
              If this message persists, please check that your Parity node is running.
            </div>
          </div>
        </div>
      </div>
    );
  }
}
