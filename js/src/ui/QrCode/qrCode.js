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

// https://github.com/cmanzana/qrcode-npm packaging the standard
// https://github.com/kazuhikoarase/qrcode-generator
import QRcodeReact from 'qr-code-react';
import React, { Component, PropTypes } from 'react';

const QROPTS = {
  CODE_TYPE:   4,
  ERROR_LEVEL: 'M',
  COLOR:       "#000000",
  BG_COLOR:    "#FFFFFF",
};

export default class QrCode extends Component {
  static propTypes = {
    className: PropTypes.string,
    margin:    PropTypes.number,
    size:      PropTypes.number,
    value:     PropTypes.string.isRequired
  };

  static defaultProps = {
    margin: 2,
    size:   4
  };

  render () {
    const { className, margin, size, value } = this.props;

    return (
      <QRcodeReact
        className={ className }
        value={ value }
        margin={ margin }
        size={ size }
        codeType={ QROPTS.CODE_TYPE }
        errorLevel={ QROPTS.ERROR_LEVEL }
        color={ QROPTS.COLOR }
        bgColor={ QROPTS.BG_COLOR }
      />
    );
  }
}
