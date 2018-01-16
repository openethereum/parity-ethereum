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
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { Button, Container, ContainerTitle, Input, ScrollableText, ShortenedHash } from '~/ui';
import { showSnackbar } from '~/redux/providers/snackbarActions';
import { newError } from '~/redux/actions';

import styles from './peers.css';

class Peers extends Component {
  static contextTypes = {
    api: PropTypes.object
  };

  static propTypes = {
    peers: PropTypes.array.isRequired,
    newError: PropTypes.func,
    showSnackbar: PropTypes.func
  };

  state = {
    action: '',
    formInput: '',
    showForm: false
  };

  getActions () {
    return [
      <Button
        key='btn_acceptNonReserved'
        label={
          <FormattedMessage
            id='peers.acceptNonReserved.label'
            defaultMessage='Accept non-reserved'
          />
        }
        onClick={ this.handleAcceptNonReserved }
      />,
      <Button
        key='btn_dropNonReserved'
        label={
          <FormattedMessage
            id='peers.dropNonReserved.label'
            defaultMessage='Drop non-reserved'
          />
        }
        onClick={ this.handleDropNonReserved }
      />,
      <Button
        key='btn_addReserved'
        label={
          <FormattedMessage
            id='peers.addReserved.label'
            defaultMessage='Add reserved'
          />
        }
        onClick={ this.handleAddReserved }
      />,
      <Button
        key='btn_removeReserved'
        label={
          <FormattedMessage
            id='peers.removeReserved.label'
            defaultMessage='Remove reserved'
          />
        }
        onClick={ this.handleRemoveReserved }
      />
    ];
  }

  render () {
    const { peers } = this.props;

    return (
      <Container>
        <ContainerTitle
          actions={ this.getActions() }
          title={
            <FormattedMessage
              id='status.peers.title'
              defaultMessage='network peers'
            />
          }
        />
        { this.renderForm() }
        <div className={ styles.peers }>
          <table>
            <thead>
              <tr>
                <th />
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.id'
                    defaultMessage='ID'
                  />
                </th>
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.remoteAddress'
                    defaultMessage='Remote Address'
                  />
                </th>
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.name'
                    defaultMessage='Name'
                  />
                </th>
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.ethHeader'
                    defaultMessage='Header (ETH)'
                  />
                </th>
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.ethDiff'
                    defaultMessage='Difficulty (ETH)'
                  />
                </th>
                <th>
                  <FormattedMessage
                    id='status.peers.table.header.caps'
                    defaultMessage='Capabilities'
                  />
                </th>
              </tr>
            </thead>
            <tbody>
              { this.renderPeers(peers) }
            </tbody>
          </table>
        </div>
      </Container>
    );
  }

  renderForm () {
    const { action, showForm } = this.state;

    if (!showForm) {
      return null;
    }

    return (
      <div className={ styles.form }>
        <div className={ styles.input }>
          <Input
            label={
              <FormattedMessage
                id='peers.form.label'
                defaultMessage='Peer enode URL'
              />
            }
            onChange={ this.handleInputChange }
          />
        </div>
        <Button
          label={
            <FormattedMessage
              id='peers.form.action.label'
              defaultMessage='{add, select, true {Add} false {}}{remove, select, true {Remove} false {}}'
              values={ {
                add: action === 'add',
                remove: action === 'remove'
              } }
            />
          }
          onClick={ this.handleConfirmForm }
        />
        <Button
          label={
            <FormattedMessage
              id='peers.form.cancel.label'
              defaultMessage='Cancel'
            />
          }
          onClick={ this.handleCancelForm }
        />
      </div>
    );
  }

  renderPeers (peers) {
    return peers.map((peer, index) => this.renderPeer(peer, index));
  }

  renderPeer (peer, index) {
    const { caps, id, name, network, protocols } = peer;

    return (
      <tr
        className={ styles.peer }
        key={ id }
      >
        <td>
          { index + 1 }
        </td>
        <td>
          <ScrollableText small text={ id } />
        </td>
        <td>
          { network.remoteAddress }
        </td>
        <td>
          { name }
        </td>
        <td>
          {
            protocols.eth
            ? <ShortenedHash data={ protocols.eth.head } />
            : null
          }
        </td>
        <td>
          {
            protocols.eth && protocols.eth.difficulty.gt(0)
            ? protocols.eth.difficulty.toExponential(16)
            : null
          }
        </td>
        <td>
          {
            caps && caps.length > 0
            ? caps.join(' - ')
            : null
          }
        </td>
      </tr>
    );
  }

  handleAcceptNonReserved = () => {
    return this.context.api.parity.acceptNonReservedPeers()
      .then(() => {
        const message = (
          <FormattedMessage
            id='peers.acceptNonReservedPeers.success'
            defaultMessage='Accepting non-reserved peers'
          />
        );

        this.props.showSnackbar(message, 3000);
      })
      .catch((error) => {
        this.props.newError(error);
      });
  };

  handleDropNonReserved = () => {
    return this.context.api.parity.dropNonReservedPeers()
      .then(() => {
        const message = (
          <FormattedMessage
            id='peers.dropNonReservedPeers.success'
            defaultMessage='Dropping non-reserved peers'
          />
        );

        this.props.showSnackbar(message, 3000);
      })
      .catch((error) => {
        this.props.newError(error);
      });
  };

  handleAddReserved = () => {
    this.setState({ showForm: true, action: 'add' });
  };

  handleRemoveReserved = () => {
    this.setState({ showForm: true, action: 'remove' });
  };

  handleInputChange = (event, value) => {
    this.setState({ formInput: value });
  };

  handleCancelForm = () => {
    this.setState({ showForm: false, action: '', formInput: '' });
  };

  handleConfirmForm = () => {
    const { action, formInput } = this.state;
    let method;

    if (action === 'add') {
      method = 'addReservedPeer';
    } else if (action === 'remove') {
      method = 'removeReservedPeer';
    }

    this.setState({ showForm: false, action: '', formInput: '' });

    if (!method) {
      return;
    }

    this.context.api.parity[method](formInput)
      .then(() => {
        const message = (
          <FormattedMessage
            id='peers.form.action.success'
            defaultMessage='Successfully {add, select, true {added} false {}}{remove, select, true {removed} false {}} a reserved peer'
            values={ {
              add: action === 'add',
              remove: action === 'remove'
            } }
          />
        );

        this.props.showSnackbar(message, 3000);
      })
      .catch((error) => {
        this.props.newError(error);
      });
  };
}

function mapStateToProps (state) {
  const handshakeRegex = /handshake/i;

  const { netPeers } = state.nodeStatus;
  const { peers = [] } = netPeers;
  const realPeers = peers
    .filter((peer) => peer.id)
    .filter((peer) => !handshakeRegex.test(peer.network.remoteAddress))
    .filter((peer) => peer.protocols.eth && peer.protocols.eth.head)
    .sort((peerA, peerB) => {
      const idComp = peerA.id.localeCompare(peerB.id);

      return idComp;
    });

  return { peers: realPeers };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError,
    showSnackbar
  }, dispatch);
}

export default connect(mapStateToProps, mapDispatchToProps)(Peers);
