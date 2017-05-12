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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { Menu } from 'semantic-ui-react';

import imagesEthcoreBlock from '~/../assets/images/parity-logo-white-no-text.svg';

import { Page } from '~/ui';
import { BackgroundIcon, EthernetIcon, VisibleIcon } from '~/ui/Icons';

import styles from './settings.css';

export default class Settings extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    children: PropTypes.object.isRequired
  }

  render () {
    const { children } = this.props;
    const hash = (window.location.hash || '').split('?')[0].split('/')[2];
    const isProxied = window.location.hostname.indexOf('.parity') !== -1;
    let proxy = null;

    if (!isProxied) {
      proxy = this.renderTab(hash, 'proxy', <EthernetIcon />);
    }

    return (
      <div>
        <Menu className={ styles.tabs } name={ hash }>
          { this.renderTab(hash, 'views', <VisibleIcon />) }
          { this.renderTab(hash, 'background', <BackgroundIcon />) }
          { proxy }
          { this.renderTab(hash, 'parity', <img src={ imagesEthcoreBlock } className={ styles.imageIcon } />) }
        </Menu>
        <Page>
          { children }
        </Page>
      </div>
    );
  }

  renderTab (hash, name, icon) {
    return (
      <Menu.Item
        className={
          hash === name
            ? styles.tabactive
            : styles.tab
        }
        icon={ icon }
        key={ name }
        content={
          <div className={ styles.menu }>
            <FormattedMessage id={ `settings.${name}.label` } />
          </div>
        }
        onClick={ this.onActivate(name) }
        name={ name }
      />
    );
  }

  onActivate = (name) => {
    const { router } = this.context;

    return (event) => {
      router.push(`/${name}`);
    };
  }
}
