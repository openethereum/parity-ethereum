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
import { Checkbox as CheckboxUI } from 'semantic-ui-react';

import { nodeOrStringProptype } from '~/util/proptypes';

export default function Checkbox ({ as, checked, className, defaultChecked, defaultIndeterminate, disabled, fitted, indeterminate, label, name, onChange, onCheck, onClick, onMouseDown, readOnly, style, tabIndex }) {
  if (onCheck) {
    onChange = onCheck;
  }

  return (
    <CheckboxUI
      as={ as }
      checked={ checked }
      className={ className }
      defaultChecked={ defaultChecked }
      defaultIndeterminate={ defaultIndeterminate }
      disabled={ disabled }
      fitted={ fitted }
      indeterminate={ indeterminate }
      label={ label }
      name={ name }
      onChange={ onChange }
      onClick={ onClick }
      onMouseDown={ onMouseDown }
      readOnly={ readOnly }
      style={ style }
      tabIndex={ tabIndex }
    />
  );
}

Checkbox.propTypes = {
  as: nodeOrStringProptype(),           // An element type to render as (string or function).
  checked: PropTypes.bool,              // Whether or not checkbox is checked.
  className: PropTypes.string,          // Additional classes.
  defaultChecked: PropTypes.bool,       // The initial value of checked.
  defaultIndeterminate: PropTypes.bool, // Whether or not checkbox is indeterminate.
  disabled: PropTypes.bool,             // A checkbox can appear disabled and be unable to change states
  fitted: PropTypes.bool,               // Removes padding for a label. Auto applied when there is no label.
  indeterminate: PropTypes.bool,        // Whether or not checkbox is indeterminate.
  label: nodeOrStringProptype(),        // The text of the associated label element.
  name: PropTypes.string,               // The HTML input name.
  onChange: PropTypes.func,             // Called when the user attempts to change the checked state.
  onCheck: PropTypes.func,              // Called when the user attempts to change the checked state.
  onClick: PropTypes.func,              // Called when the checkbox or label is clicked.
  onMouseDown: PropTypes.func,          // Called when the user presses down on the mouse.
  readOnly: PropTypes.bool,             // Format as a radio element. This means it is an exclusive option.
  style: PropTypes.object,              // Style format.
  tabIndex: PropTypes.number            // A checkbox can receive focus.
};
