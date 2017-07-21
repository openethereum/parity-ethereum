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

import React from 'react';
import PropTypes from 'prop-types';

import Button from '../../Button';
import { LockedIcon, UnlockedIcon } from '../../Icons';

import styles from '../vaultCard.css';

export default function Buttons ({ buttons, hideButtons, vault }) {
  const { isOpen } = vault;

  if (hideButtons) {
    return null;
  }

  return (
    <div className={ styles.buttons }>
      <Button
        className={ styles.status }
        disabled
        icon={
          isOpen
            ? <UnlockedIcon />
            : <LockedIcon />
        }
        key='status'
      />
      { buttons }
    </div>
  );
}

Buttons.propTypes = {
  buttons: PropTypes.array,
  hideButtons: PropTypes.bool,
  vault: PropTypes.object.isRequired
};
