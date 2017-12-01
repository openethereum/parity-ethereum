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
    account: PropTypes.object.isRequired,
    isDefault: PropTypes.bool,
    onClick: PropTypes.func
  }

  handleClick = () => {
    this.props.onClick(this.props.account.address);
  }

  render () {
    const { account, isDefault } = this.props;

    return (
      <List.Item
        onClick={ this.handleClick }
        disabled={ isDefault }
      >
        <Image avatar>
          <div className={ styles.avatarWrapper }>
            <IdentityIcon address={ account.address }
              alt={ account.address }
              className={ isDefault ? styles.bigAvatar : '' }
            />
          </div>
        </Image>
        <List.Content className={ isDefault ? styles.defaultContent : '' }>
          <List.Header>
            {account.name}
          </List.Header>
          {account.address}
          {isDefault && <p className={ styles.description }>{account.meta.description}</p>}
        </List.Content>
      </List.Item>
    );
  }
}

export default AccountItem;
