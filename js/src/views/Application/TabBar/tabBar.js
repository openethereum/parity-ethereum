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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';

import { Badge, Tooltip } from '../../../ui';

import styles from './tabBar.css';
import imagesEthcoreBlock from '../../../../assets/images/parity-logo-white-no-text.svg';

const TABMAP = {
  accounts: 'account',
  addresses: 'address',
  apps: 'app',
  contracts: 'contract',
  deploy: 'contract'
};

class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    pending: PropTypes.array,
    isTest: PropTypes.bool,
    netChain: PropTypes.string,
    settings: PropTypes.object.isRequired
  }

  state = {
    activeRoute: '/accounts'
  }

  render () {
    return (
      <Toolbar
        className={ styles.toolbar }>
        { this.renderLogo() }
        { this.renderTabs() }
        { this.renderLast() }
      </Toolbar>
    );
  }

  renderLogo () {
    return (
      <ToolbarGroup>
        <div className={ styles.logo }>
          <img src={ imagesEthcoreBlock } />
        </div>
      </ToolbarGroup>
    );
  }

  renderLast () {
    return (
      <ToolbarGroup>
        <div className={ styles.last }>
          <div></div>
        </div>
      </ToolbarGroup>
    );
  }

  renderTabs () {
    const { settings } = this.props;
    const windowHash = (window.location.hash || '').split('?')[0].split('/')[1];
    const hash = TABMAP[windowHash] || windowHash;

    const items = Object.keys(settings.views)
      .filter((id) => settings.views[id].fixed || settings.views[id].active)
      .map((id) => {
        const view = settings.views[id];
        let label = this.renderLabel(view.label);
        let body = null;

        if (id === 'accounts') {
          body = (
            <Tooltip className={ styles.tabbarTooltip } text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          );
        } else if (id === 'signer') {
          label = this.renderSignerLabel(label);
        } else if (id === 'status') {
          label = this.renderStatusLabel(label);
        }

        return (
          <Tab
            className={ hash === view.value ? styles.tabactive : '' }
            value={ view.value }
            icon={ view.icon }
            key={ id }
            label={ label }
            onActive={ this.onActivate(view.route) }>
            { body }
          </Tab>
        );
      });

    return (
      <Tabs
        className={ styles.tabs }
        value={ hash }>
        { items }
      </Tabs>
    );
  }

  renderLabel = (name, bubble) => {
    return (
      <div className={ styles.label }>
        { name }
        { bubble }
      </div>
    );
  }

  renderSignerLabel = (label) => {
    const { pending } = this.props;
    let bubble = null;

    if (pending && pending.length) {
      bubble = (
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pending.length } />
      );
    }

    return this.renderLabel(label, bubble);
  }

  renderStatusLabel = (label) => {
    // const { isTest, netChain } = this.props;
    // const bubble = (
    //   <Badge
    //     color={ isTest ? 'red' : 'default' }
    //     className={ styles.labelBubble }
    //     value={ isTest ? 'TEST' : netChain } />
    //   );

    return this.renderLabel(label, null);
  }

  onActivate = (activeRoute) => {
    const { router } = this.context;

    return (event) => {
      router.push(activeRoute);
      this.setState({ activeRoute });
    };
  }
}

function mapStateToProps (state) {
  const { settings } = state;

  return { settings };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TabBar);
