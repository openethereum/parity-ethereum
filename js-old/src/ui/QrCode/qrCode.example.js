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

import React, { Component } from 'react';

import PlaygroundExample from '~/playground/playgroundExample';

import QrCode from './';

export default class QrCodeExample extends Component {
  render () {
    return (
      <div>
        <PlaygroundExample name='Simple QRCode'>
          <QrCode
            value='this is a test'
          />
        </PlaygroundExample>

        <PlaygroundExample name='Simple QRCode with margin'>
          <QrCode
            margin={ 10 }
            value='this is a test'
          />
        </PlaygroundExample>

        <PlaygroundExample name='Ethereum Address QRCode'>
          <QrCode
            margin={ 10 }
            value='0x8c30393085C8C3fb4C1fB16165d9fBac5D86E1D9'
          />
        </PlaygroundExample>

        <PlaygroundExample name='Bitcoin Address QRCode'>
          <QrCode
            margin={ 10 }
            value='3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy'
          />
        </PlaygroundExample>

        <PlaygroundExample name='Big QRCode'>
          <QrCode
            size={ 10 }
            value='this is a test'
          />
        </PlaygroundExample>
      </div>
    );
  }
}
