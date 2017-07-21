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

import { nodeOrStringProptype } from '@parity/shared/util/proptypes';

import { CloseIcon } from '~/ui/Icons';

import styles from './chip.css';

export default function Chip ({ className, isDisabled, isFocused, label, onClick, onDelete }) {
  return (
    <div
      className={ `${styles.chip} ${isDisabled && styles.disabled} ${isFocused && styles.focus} ${className}` }
      onTouchTap={ onClick }
    >
      <div className={ styles.label }>
        { label }
      </div>
      <CloseIcon
        className={ styles.delete }
        onTouchTap={ onDelete }
      />
    </div>
  );
}

Chip.propTypes = {
  className: PropTypes.string,
  isDisabled: PropTypes.bool,
  isFocused: PropTypes.bool,
  label: nodeOrStringProptype(),
  onClick: PropTypes.func,
  onDelete: PropTypes.func
};
