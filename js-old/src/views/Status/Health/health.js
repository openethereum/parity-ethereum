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

import { Container, ContainerTitle, StatusIndicator } from '~/ui';

import grid from '../NodeStatus/nodeStatus.css';

const HealthItem = (props) => {
  const status = props.item.status || 'needsAttention';

  return (
    <div>
      <h3>
        <StatusIndicator
          id={ props.id }
          title={ [
            (<div>{ props.item.message }</div>)
          ] }
          status={ status }
        />
        { props.title }
        <small>&nbsp;({ props.details })</small>
      </h3>
      <p>
        { status !== 'ok' ? props.item.message : '' }
      </p>
    </div>
  );
};

HealthItem.propTypes = {
  id: PropTypes.string.isRequired,
  title: PropTypes.node.isRequired,
  details: PropTypes.oneOfType([
    PropTypes.string,
    PropTypes.node
  ]).isRequired,
  item: PropTypes.object.isRequired
};

class Health extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    peers: PropTypes.object.isRequired,
    sync: PropTypes.object.isRequired,
    time: PropTypes.object.isRequired
  };

  state = {};

  render () {
    const { peers, sync, time } = this.props;
    const [yes, no] = [(
      <FormattedMessage
        id='status.health.yes'
        defaultMessage='yes'
      />
    ), (
      <FormattedMessage
        id='status.health.no'
        defaultMessage='no'
      />
    )];

    return (
      <Container>
        <ContainerTitle
          title={
            <div>
              <FormattedMessage
                id='status.health.title'
                defaultMessage='Node Health'
              />
            </div>
          }
        />
        <div className={ grid.container }>
          <div className={ grid.row }>
            <div className={ grid.col4 }>
              <HealthItem
                id='status.health.sync'
                title={
                  <FormattedMessage
                    id='status.health.sync'
                    defaultMessage='Chain Synchronized'
                  />
                }
                details={ !sync.details ? yes : no }
                item={ sync }
              />
            </div>
            <div className={ grid.col4 }>
              <HealthItem
                id='status.health.peers'
                title={
                  <FormattedMessage
                    id='status.health.peers'
                    defaultMessage='Connected Peers'
                  />
                }
                details={ (peers.details || []).join('/') }
                item={ peers }
              />
            </div>
            <div className={ grid.col4 }>
              <HealthItem
                id='status.health.time'
                title={
                  <FormattedMessage
                    id='status.health.time'
                    defaultMessage='Time Synchronized'
                  />
                }
                details={ `${time.details || 0} ms` }
                item={ time }
              />
            </div>
          </div>
        </div>
      </Container>
    );
  }
}

function mapStateToProps (state) {
  return state.nodeStatus.health;
}

export default connect(
  mapStateToProps,
  null
)(Health);
