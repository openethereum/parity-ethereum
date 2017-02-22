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

import { IdentityIcon } from '~/ui';
import { AccountsIcon, DoneIcon, FileIcon, FileUploadIcon, KeyboardIcon, KeyIcon, MembershipIcon } from '~/ui/Icons';

import { STAGE_INFO } from '../store';

export default class TypeIcon extends Component {
  static propTypes = {
    store: PropTypes.object.isRequired,
    type: PropTypes.string
  }

  render () {
    const { store, type } = this.props;
    const { address, createType, stage } = store;

    if (stage === STAGE_INFO) {
      return (type || createType) === 'fromGeth'
        ? (
          <DoneIcon />
        )
        : (
          <IdentityIcon
            address={ address }
            center
          />
        );
    }

    switch (type || createType) {
      case 'fromGeth':
        return <FileUploadIcon />;

      case 'fromPhrase':
        return <KeyboardIcon />;

      case 'fromRaw':
        return <KeyIcon />;

      case 'fromJSON':
        return <FileIcon />;

      case 'fromPresale':
        return <MembershipIcon />;

      case 'fromNew':
      default:
        return <AccountsIcon />;
    }
  }
}
