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
import { Tab as MUITab } from 'material-ui/Tabs';
import { isEqual } from 'lodash';

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

class Tab extends Component {
  static propTypes = {
    active: PropTypes.bool,
    view: PropTypes.object,
    children: PropTypes.node,
    pendings: PropTypes.number,
    onChange: PropTypes.func
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
        onClick={ this.handleClick }
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

  handleClick = () => {
    const { onChange, view } = this.props;
    onChange(view);
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

  state = {
    activeViewId: ''
  };

  setActiveView (props = this.props) {
    const { hash, views } = props;
    const view = views.find((view) => view.value === hash);

    this.setState({ activeViewId: view.id });
  }

  componentWillMount () {
    this.setActiveView();
  }

  componentWillReceiveProps (nextProps) {
    if (nextProps.hash !== this.props.hash) {
      this.setActiveView(nextProps);
    }
  }

  shouldComponentUpdate (nextProps, nextState) {
    const prevViews = this.props.views.map((v) => v.id).sort();
    const nextViews = nextProps.views.map((v) => v.id).sort();

    return (nextProps.hash !== this.props.hash) ||
      (nextProps.pending.length !== this.props.pending.length) ||
      (nextState.activeViewId !== this.state.activeViewId) ||
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
    const { views, pending } = this.props;
    const { activeViewId } = this.state;

    const items = views
      .map((view, index) => {
        const body = (view.id === 'accounts')
          ? (
          <Tooltip className={ styles.tabbarTooltip } text='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view' />
          )
          : null;

        const active = activeViewId === view.id;

        return (
          <Tab
            active={ active }
            view={ view }
            onChange={ this.onChange }
            key={ view.id }
            pendings={ pending.length }
          >
            { body }
          </Tab>
        );
      });

    return (
      <div
        className={ styles.tabs }
        onChange={ this.onChange }>
        { items }
      </div>
    );
  }

  onChange = (view) => {
    const { router } = this.context;

    router.push(view.route);
    this.setState({ activeViewId: view.id });
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
