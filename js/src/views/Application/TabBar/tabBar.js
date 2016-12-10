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
import { Link } from 'react-router';
import { bindActionCreators } from 'redux';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { Tab as MUITab } from 'material-ui/Tabs';
import { isEqual } from 'lodash';

import { Badge, Tooltip } from '~/ui';

import styles from './tabBar.css';
import imagesEthcoreBlock from '../../../../assets/images/parity-logo-white-no-text.svg';

const TABMAP = {
  accounts: 'account',
  wallet: 'account',
  addresses: 'address',
  apps: 'app',
  contracts: 'contract',
  deploy: 'contract'
};

class Tab extends Component {
  static propTypes = {
    active: PropTypes.bool,
    view: PropTypes.object,
    children: PropTypes.node,
    pendings: PropTypes.number
  };

  shouldComponentUpdate (nextProps) {
    return nextProps.active !== this.props.active ||
      (nextProps.view.id === 'signer' && nextProps.pendings !== this.props.pendings);
  }

  render () {
    const { active, view, children } = this.props;

    const label = this.getLabel(view);

    return (
      <MUITab
        className={ active ? styles.tabactive : '' }
        selected={ active }
        icon={ view.icon }
        label={ label }
      >
        { children }
      </MUITab>
    );
  }

  getLabel (view) {
    const { label } = view;

    if (view.id === 'signer') {
      return this.renderSignerLabel(label);
    }

    if (view.id === 'status') {
      return this.renderStatusLabel(label);
    }

    return this.renderLabel(label);
  }

  renderLabel (name, bubble) {
    return (
      <div className={ styles.label }>
        { name }
        { bubble }
      </div>
    );
  }

  renderSignerLabel (label) {
    const { pendings } = this.props;

    if (pendings) {
      const bubble = (
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pendings } />
      );

      return this.renderLabel(label, bubble);
    }

    return this.renderLabel(label);
  }

  renderStatusLabel (label) {
    // const { isTest, netChain } = this.props;
    // const bubble = (
    //   <Badge
    //     color={ isTest ? 'red' : 'default' }
    //     className={ styles.labelBubble }
    //     value={ isTest ? 'TEST' : netChain } />
    //   );

    return this.renderLabel(label, null);
  }
}

class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    views: PropTypes.array.isRequired,
    hash: PropTypes.string.isRequired,
    pending: PropTypes.array,
    isTest: PropTypes.bool,
    netChain: PropTypes.string
  };

  static defaultProps = {
    pending: []
  };

  shouldComponentUpdate (nextProps, nextState) {
    const prevViews = this.props.views.map((v) => v.id).sort();
    const nextViews = nextProps.views.map((v) => v.id).sort();

    return (nextProps.hash !== this.props.hash) ||
      (nextProps.pending.length !== this.props.pending.length) ||
      (!isEqual(prevViews, nextViews));
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
          <img src={ imagesEthcoreBlock } height={ 28 } />
        </div>
      </ToolbarGroup>
    );
  }

  renderLast () {
    return (
      <ToolbarGroup>
        <div className={ styles.last }>
          <div />
        </div>
      </ToolbarGroup>
    );
  }

  renderTabs () {
    const { views, pending } = this.props;

    const items = views
      .map((view, index) => {
        const body = (view.id === 'accounts')
          ? (
            <Tooltip className={ styles.tabbarTooltip } text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          )
          : null;

        return (
          <Link
            key={ view.id }
            to={ view.route }
            activeClassName={ styles.tabactive }
            className={ styles.tabLink }
          >
            <Tab
              view={ view }
              pendings={ pending.length }
            >
              { body }
            </Tab>
          </Link>
        );
      });

    return (
      <div className={ styles.tabs }>
        { items }
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { views } = state.settings;

  const filteredViews = Object
    .keys(views)
    .filter((id) => views[id].fixed || views[id].active)
    .map((id) => ({
      ...views[id],
      id
    }));

  const windowHash = (window.location.hash || '').split('?')[0].split('/')[1];
  const hash = TABMAP[windowHash] || windowHash;

  return { views: filteredViews, hash };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(TabBar);
