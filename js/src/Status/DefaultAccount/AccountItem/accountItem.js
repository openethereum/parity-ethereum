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

import React, { PureComponent } from 'react';
import PropTypes from 'prop-types';

import Image from 'semantic-ui-react/dist/commonjs/elements/Image';
import List from 'semantic-ui-react/dist/commonjs/elements/List';
import IdentityIcon from '@parity/ui/lib/IdentityIcon';
import styles from './accountItem.css';

class AccountItem extends PureComponent {
  static propTypes = {
    address: PropTypes.string.isRequired,
    isDefault: PropTypes.bool,
    name: PropTypes.string.isRequired,
    onClick: PropTypes.func
  }

  handleClick = () => {
    this.props.onClick(this.props.address);
  }

  render () {
    const { address, name, isDefault } = this.props;

    return (
      <List.Item
        key={ address }
        onClick={ this.handleClick }
        disabled={ isDefault }
        className={ isDefault ? styles.isDefault : '' }
      >
        <Image avatar as={ IdentityIcon } address={ address } alt={ address } />
        <List.Content>
          <List.Header>{name}</List.Header>
          {address}
        </List.Content>
      </List.Item>
    );
  }
}

export default AccountItem;
