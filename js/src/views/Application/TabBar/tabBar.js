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
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tabs, Tab } from 'material-ui/Tabs';

import { defaultViews } from '../../Settings';
import { Badge, ParityBackground, Tooltip } from '../../../ui';

import styles from './tabBar.css';
import imagesEthcoreBlock from '../../../images/ethcore-block.png';

const TABMAP = {
  accounts: 'account',
  addresses: 'address',
  apps: 'app',
  contracts: 'contract'
};

export default class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  }

  static propTypes = {
    pending: PropTypes.array,
    isTest: PropTypes.bool,
    netChain: PropTypes.string
  }

  state = {
    accountsVisible: true,
    addressesVisible: true,
    appsVisible: true,
    statusVisible: true,
    signerVisible: true,
    activeRoute: '/accounts'
  }

  constructor () {
    super();

    defaultViews.accounts.body = <Tooltip className={ styles.tabbarTooltip } text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />;
    defaultViews.signer.renderLabel = this.renderSignerLabel;
    defaultViews.status.renderLabel = this.renderStatusLabel;
  }

  componentDidMount () {
    this.loadViews();
  }

  render () {
    return (
      <ParityBackground>
        <Toolbar
          className={ styles.toolbar }>
          { this.renderLogo() }
          { this.renderTabs() }
          { this.renderLast() }
        </Toolbar>
      </ParityBackground>
    );
  }

  renderLogo () {
    return (
      <ToolbarGroup>
        <div className={ styles.logo }>
          <img src={ imagesEthcoreBlock } />
          <div>Parity</div>
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
    const windowHash = (window.location.hash || '').split('?')[0].split('/')[1];
    const hash = TABMAP[windowHash] || windowHash;

    const items = Object.keys(defaultViews)
      .filter((id) => {
        const tab = defaultViews[id];
        const isFixed = tab.fixed;
        const isVisible = this.state[this.visibleId(id)];

        return isFixed || isVisible;
      })
      .map((id) => {
        const tab = defaultViews[id];
        const onActivate = () => this.onActivate(tab.route);

        return (
          <Tab
            className={ hash === tab.value ? styles.tabactive : '' }
            value={ tab.value }
            icon={ tab.icon }
            key={ id }
            label={ tab.renderLabel ? tab.renderLabel(tab.label) : this.renderLabel(tab.label) }
            onActive={ onActivate }>
            { tab.body }
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
    const { isTest, netChain } = this.props;
    const bubble = (
      <Badge
        color={ isTest ? 'red' : 'default' }
        className={ styles.labelBubble }
        value={ isTest ? 'TEST' : netChain } />
      );

    return this.renderLabel(label, bubble);
  }

  visibleId (id) {
    return `${id}Visible`;
  }

  onActivate = (activeRoute) => {
    const { router } = this.context;

    router.push(activeRoute);
    this.setState({ activeRoute });
  }

  toggleMenu = (event, menu) => {
    const id = menu.props['data-id'];
    const toggle = this.visibleId(id);
    const isActive = this.state[toggle];

    if (defaultViews[id].fixed) {
      return;
    }

    this.setState({
      [toggle]: !isActive
    }, this.saveViews);
  }

  loadViews () {
    const state = {};
    const data = defaultViews.load();

    Object.keys(data).forEach((id) => {
      state[this.visibleId(id)] = data[id].active;
    });

    this.setState(state, this.saveViews);
  }

  saveViews = () => {
    const data = {};

    Object.keys(data).forEach((id) => {
      data[id] = { active: this.state[this.visibleId(id)] };
    });

    defaultViews.save(data);
  }
}
