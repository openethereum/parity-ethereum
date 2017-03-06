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
import { connect } from 'react-redux';
import { Link } from 'react-router';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';
import { isEqual } from 'lodash';

import { Tooltip } from '~/ui';

import Tab from './Tab';
import styles from './tabBar.css';

class TabBar extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    pending: PropTypes.array,
    views: PropTypes.array.isRequired
  };

  static defaultProps = {
    pending: []
  };

  render () {
    return (
      <Toolbar className={ styles.toolbar }>
        <ToolbarGroup className={ styles.first }>
          <div />
        </ToolbarGroup>
        <div className={ styles.tabs }>
          { this.renderTabItems() }
          <Tooltip
            className={ styles.tabbarTooltip }
            text={
              <FormattedMessage
                id='tabBar.tooltip.overview'
                defaultMessage='navigate between the different parts and views of the application, switching between an account view, token view and distributed application view'
              />
            }
          />
        </div>
        <ToolbarGroup className={ styles.last }>
          <div />
        </ToolbarGroup>
      </Toolbar>
    );
  }

  renderTabItems () {
    const { views, pending } = this.props;

    return views.map((view, index) => {
      return (
        <Link
          activeClassName={ styles.tabactive }
          className={ styles.tabLink }
          key={ view.id }
          to={ view.route }
        >
          <Tab
            pendings={ pending.length }
            view={ view }
          />
        </Link>
      );
    });
  }
}

function mapStateToProps (initState) {
  const { views } = initState.settings;

  let filteredViewIds = Object
    .keys(views)
    .filter((id) => views[id].fixed || views[id].active);

  let filteredViews = filteredViewIds.map((id) => ({
    ...views[id],
    id
  }));

  return (state) => {
    const { views } = state.settings;

    const viewIds = Object
      .keys(views)
      .filter((id) => views[id].fixed || views[id].active);

    if (isEqual(viewIds, filteredViewIds)) {
      return { views: filteredViews };
    }

    filteredViewIds = viewIds;
    filteredViews = viewIds.map((id) => ({
      ...views[id],
      id
    }));

    return { views: filteredViews };
  };
}

export default connect(
  mapStateToProps,
  null
)(TabBar);
