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
import { Dropdown as SemanticDropdown } from 'semantic-ui-react';

import LabelComponent from '../labelComponent';

import styles from './dropdown.css';

// FIXME: Currently does not display the selected icon alongside
export default function Dropdown ({ className, disabled = false, fullWidth = true, hint, label, onChange, options, value }) {
  return (
    <LabelComponent label={ label }>
      <SemanticDropdown
        className={ `${styles.dropdown} ${className}` }
        disabled={ disabled }
        fluid={ fullWidth }
        onChange={ onChange }
        options={ options }
        placeholder={ hint }
        scrolling
        search
        selection
        value={ value }
      />
    </LabelComponent>
  );
}

Dropdown.propTypes = {
  className: PropTypes.string,
  disabled: PropTypes.bool,          // A disabled dropdown menu or item does not allow user interaction.
  fullWidth: PropTypes.bool,         // A dropdown can take the full width of its parent.
  hint: PropTypes.string,            // Placeholder text.
  label: PropTypes.node,
  name: PropTypes.func,              // Name of the hidden input which holds the value.
  onChange: PropTypes.func,          // Called when the user attempts to change the value.
  options: PropTypes.any,            // Array of Dropdown.Item props e.g. `{ text: '', value: '' }`
  value: PropTypes.any               // Current value. Creates a controlled component.
};
