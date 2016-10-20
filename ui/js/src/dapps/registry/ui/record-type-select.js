// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import DropDownMenu from 'material-ui/DropDownMenu';
import MenuItem from 'material-ui/MenuItem';

export default (value, onSelect, className = '') => (
  <DropDownMenu className={ className } value={ value } onChange={ onSelect }>
    <MenuItem value='A' primaryText='A – Ethereum address' />
    <MenuItem value='IMG' primaryText='IMG – hash of a picture in the blockchain' />
    <MenuItem value='CONTENT' primaryText='CONTENT – hash of a data in the blockchain' />
  </DropDownMenu>
);
