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
