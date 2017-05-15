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

import { Page, Tabs } from '@parity/ui';
import { BackgroundIcon, EthernetIcon, VisibleIcon } from '@parity/ui/Icons';

import imagesEthcoreBlock from '~/../assets/images/parity-logo-white-no-text.svg';

import styles from './settings.css';

const TABS = ['views', 'background', 'proxy', 'parity'];

export default class Settings extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    children: PropTypes.object.isRequired
  }

  render () {
    const { children } = this.props;
    const hash = (window.location.hash || '').split('?')[0].split('/')[1];
    const isProxied = window.location.hostname.indexOf('.parity') !== -1;

    return (
      <div>
        <Tabs
          activeTab={ TABS.indexOf(hash) }
          className={ styles.tabs }
          onChange={ this.onActivate }
          tabs={ [
            {
              icon: <VisibleIcon />,
              label: <FormattedMessage id='settings.views.label' />
            },
            {
              icon: <BackgroundIcon />,
              label: <FormattedMessage id='settings.background.label' />
            },
            isProxied
              ? null
              : {
                icon: <EthernetIcon />,
                label: <FormattedMessage id='settings.proxy.label' />
              },
            {
              icon: <img src={ imagesEthcoreBlock } className={ styles.imageIcon } />,
              label: <FormattedMessage id='settings.parity.label' />
            }
          ] }
        />
        <Page>
          { children }
        </Page>
      </div>
    );
  }

  onActivate = (event, tabIndex) => {
    const { router } = this.context;

    router.push(`/${TABS[tabIndex]}`);
  }
}
