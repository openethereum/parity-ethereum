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

import React, { PropTypes } from 'react';
import Reader from 'react-qr-reader';

import styles from './qrScan.css';

const SCAN_DELAY = 100;
const SCAN_STYLE = {
  display: 'inline-block',
  width: '30em'
};

export default function QrScan ({ onError, onScan }) {
  return (
    <div className={ styles.qr }>
      <Reader
        delay={ SCAN_DELAY }
        onError={ onError }
        onScan={ onScan }
        style={ SCAN_STYLE }
      />
    </div>
  );
}

QrScan.propTypes = {
  onError: PropTypes.func,
  onScan: PropTypes.func.isRequired
};

QrScan.defaultProps = {
  onError: (error) => {
    console.log('QrScan', error);
  }
};
