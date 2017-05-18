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

import { parseI18NString } from '@parity/shared/util/messages';

import LabelWrapper from '../LabelWrapper';

import styles from './dropdown.css';

const NAME_ID = ' ';

// FIXME: Currently does not display the selected icon alongside
export default function Dropdown ({ className, disabled = false, error, fullWidth = true, hint, label, onBlur, onChange, onKeyDown, options, text, value }, context) {
  const _onChange = (event, { value }) => onChange(event, value);

  return (
    <LabelWrapper label={ label }>
      <SemanticDropdown
        className={ `${styles.dropdown} ${className}` }
        disabled={ disabled }
        error={ !!error }
        fluid={ fullWidth }
        id={ NAME_ID }
        name={ NAME_ID }
        onBlur={ onBlur }
        onChange={ _onChange }
        onKeyDown={ onKeyDown }
        options={ options }
        placeholder={ parseI18NString(context, hint) }
        scrolling
        search
        selection
        text={ parseI18NString(context, text) }
        value={ value }
      />
    </LabelWrapper>
  );
}

Dropdown.contextTypes = {
  intl: PropTypes.object.isRequired
};

Dropdown.propTypes = {
  children: PropTypes.node,
  className: PropTypes.string,
  disabled: PropTypes.bool,          // A disabled dropdown menu or item does not allow user interaction.
  error: PropTypes.any,
  fullWidth: PropTypes.bool,         // A dropdown can take the full width of its parent.
  hint: PropTypes.node,            // Placeholder text.
  label: PropTypes.node,
  name: PropTypes.func,              // Name of the hidden input which holds the value.
  onChange: PropTypes.func,          // Called when the user attempts to change the value.
  onBlur: PropTypes.func,
  onKeyDown: PropTypes.func,
  options: PropTypes.any,            // Array of Dropdown.Item props e.g. `{ text: '', value: '' }`
  text: PropTypes.any,
  value: PropTypes.any,               // Current value. Creates a controlled component.
  values: PropTypes.array
};
