/* Copyright 2015-2017 Parity Technologies (UK) Ltd.
/* This file is part of Parity.
/*
/* Parity is free software: you can redistribute it and/or modify
/* it under the terms of the GNU General Public License as published by
/* the Free Software Foundation, either version 3 of the License, or
/* (at your option) any later version.
/*
/* Parity is distributed in the hope that it will be useful,
/* but WITHOUT ANY WARRANTY; without even the implied warranty of
/* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
/* GNU General Public License for more details.
/*
/* You should have received a copy of the GNU General Public License
/* along with Parity.  If not, see <http://www.gnu.org/licenses/>.
*/

/** USAGE:
  Options = [
    {
      text: 'Jenny Hess',
      value: 'Jenny Hess',
      image: { avatar: true, src: '/assets/images/avatar/small/jenny.jpg' },
    },
   ...
  ]

  <Dropdown placeholder='Select Friend' fluid selection options={Options} />
**/

import React, { PropTypes } from 'react';
import { Dropdown as DropdownUI } from 'semantic-ui-react';

export default function Dropdown ({ defaultValue, disabled, fluid, icon, name, onChange, onClick, onClose, onFocus, options, placeholder, scrolling, search, selection, text, value }) {
  return (
    <DropdownUI
      defaultValue={ defaultValue }
      disabled={ disabled }
      fluid={ fluid }
      icon={ icon }
      name={ name }
      onChange={ onChange }
      onClick={ onClick }
      onClose={ onClose }
      onFocus={ onFocus }
      options={ options }
      placeholder={ placeholder }
      scrolling={ scrolling }
      search={ search }
      selection={ selection }
      text={ text }
      value={ value }
    />
  );
}

Dropdown.propTypes = {
  defaultValue: PropTypes.number,    // Initial value via index
  disabled: PropTypes.bool,          // A disabled dropdown menu or item does not allow user interaction.
  fluid: PropTypes.bool,             // A dropdown can take the full width of its parent
  icon: PropTypes.any,               // Shorthand for Icon.
  name: PropTypes.func,              // Name of the hidden input which holds the value.
  onChange: PropTypes.func,          // Called when the user attempts to change the value.
  onClick: PropTypes.func,           // Called on click.
  onClose: PropTypes.func,           // Called on close.
  onFocus: PropTypes.func,           // Called on focus.
  options: PropTypes.any,            // Array of Dropdown.Item props e.g. `{ text: '', value: '' }`
  placeholder: PropTypes.string,     // Placeholder text.
  scrolling: PropTypes.bool,         // A dropdown can have its menu scroll.
  search: PropTypes.bool,            // A selection dropdown can allow a user to search through a large list of choices.
  selection: PropTypes.any,          // A dropdown can be used to select between choices in a form.
  text: PropTypes.string,            // The text displayed in the dropdown, usually for the active item.
  value: PropTypes.any               // Current value. Creates a controlled component.
};

Dropdown.defaultProps = {
  disabled: false
};
