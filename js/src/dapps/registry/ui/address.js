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
import renderHash from './hash';
import IdentityIcon from '../IdentityIcon';

const container = {
  display: 'inline-block',
  verticalAlign: 'middle',
  height: '24px'
};
const align = {
  display: 'inline-block',
  verticalAlign: 'top',
  lineHeight: '24px'
};

export default (address, accounts, contacts, shortenHash = true) => {
  let caption;
  if (accounts[address]) {
    caption = (<abbr title={ address } style={ align }>{ accounts[address].name }</abbr>);
  } else if (contacts[address]) {
    caption = (<abbr title={ address } style={ align }>{ contacts[address].name }</abbr>);
  } else {
    caption = (<code style={ align }>{ shortenHash ? renderHash(address) : address }</code>);
  }
  return (
    <div style={ container }>
      <IdentityIcon address={ address } style={ align } />
      { caption }
    </div>
  );
};
