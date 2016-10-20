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

// 0xecf69634885f27a8f78161e530f15a8d3b57d39e755c222c92cf297b6e25aaaa

import React, { Component, PropTypes } from 'react';
import { Tab, Tabs } from 'material-ui';
import ActionSettingsEthernet from 'material-ui/svg-icons/action/settings-ethernet';
import ImageBlurOn from 'material-ui/svg-icons/image/blur-on';
import ImageRemoveRedEye from 'material-ui/svg-icons/image/remove-red-eye';

import { Actionbar, Page } from '../../ui';

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

    return (
      <div className={ styles.layout }>
        <Actionbar title='settings' className={ styles.bar }>
          { this.renderTabs() }
        </Actionbar>
        <Page>
          { children }
        </Page>
      </div>
    );
  }

  renderTabs () {
    const hash = (window.location.hash || '').split('?')[0].split('/')[2];
    const isProxied = window.location.hostname.indexOf('.parity') !== -1;
    let proxy = null;

    if (!isProxied) {
      proxy = (
        <Tab
          className={ hash === 'proxy' ? styles.tabactive : styles.tab }
          value='proxy'
          key='proxy'
          icon={ <ActionSettingsEthernet /> }
          label={ <div className={ styles.menu }>proxy</div> }
          onActive={ this.onActivate('proxy') } />
      );
    }

    return (
      <Tabs className={ styles.tabs } value={ hash }>
        <Tab
          className={ hash === 'views' ? styles.tabactive : styles.tab }
          value='views'
          key='views'
          icon={ <ImageRemoveRedEye /> }
          label={ <div className={ styles.menu }>views</div> }
          onActive={ this.onActivate('views') } />
        <Tab
          className={ hash === 'background' ? styles.tabactive : styles.tab }
          value='background'
          key='background'
          icon={ <ImageBlurOn /> }
          label={ <div className={ styles.menu }>background</div> }
          onActive={ this.onActivate('background') } />
        { proxy }
      </Tabs>
    );
  }

  onActivate = (section) => {
    const { router } = this.context;

    return (event) => {
      router.push(`/settings/${section}`);
    };
  }
}
