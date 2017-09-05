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

import React, { Component, PropTypes } from 'react';

import { AccountsIcon, DoneIcon, FileIcon, FileUploadIcon, KeyboardIcon, KeyIcon, MembershipIcon, PhoneLock } from '~/ui/Icons';

import { STAGE_INFO } from '../store';

export default class TypeIcon extends Component {
  static propTypes = {
    className: PropTypes.string,
    createStore: PropTypes.object.isRequired,
    type: PropTypes.string
  }

  render () {
    const { className, createStore, type } = this.props;
    const { createType, stage } = createStore;

    if (stage === STAGE_INFO) {
      return <DoneIcon className={ className } />;
    }

    switch (type || createType) {
      case 'fromGeth':
        return <FileUploadIcon className={ className } />;

      case 'fromJSON':
        return <FileIcon className={ className } />;

      case 'fromPhrase':
        return <KeyboardIcon className={ className } />;

      case 'fromPresale':
        return <MembershipIcon className={ className } />;

      case 'fromQr':
        return <PhoneLock className={ className } />;

      case 'fromRaw':
        return <KeyIcon className={ className } />;

      case 'fromNew':
      default:
        return <AccountsIcon className={ className } />;
    }
  }
}
