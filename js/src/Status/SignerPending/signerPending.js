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

import React, { Component } from 'react';
import { observer } from 'mobx-react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import IdentityIcon from '@parity/ui/lib/IdentityIcon';
import Button from 'semantic-ui-react/dist/commonjs/elements/Button';
import Container from 'semantic-ui-react/dist/commonjs/elements/Container';
import Header from 'semantic-ui-react/dist/commonjs/elements/Header';
import Icon from 'semantic-ui-react/dist/commonjs/elements/Icon';
import Image from 'semantic-ui-react/dist/commonjs/elements/Image';
import Label from 'semantic-ui-react/dist/commonjs/elements/Label';
import List from 'semantic-ui-react/dist/commonjs/elements/List';
import Popup from 'semantic-ui-react/dist/commonjs/modules/Popup';

import Store from './store';
import ParityBarStore from '../../ParityBar/store';
import styles from './signerPending.css';

@observer
class SignerPending extends Component {
  static propTypes = {}

  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  state = {
    isOpen: false
  }

  store = Store.get(this.context.api)
  parityBarStore = ParityBarStore.get();

  handleViewRequest = () => {
    this.parityBarStore.toggleOpenSigner();
    this.handleClose();
  }

  handleOpen = () => {
    this.setState({ isOpen: true });
  }

  handleClose = () => {
    this.setState({ isOpen: false });
  }

  renderEtherValue (value) {
    const { api } = this.context;
    const ether = api.util.fromWei(value);

    return (
      <span>
        {ether.toFormat(5)}<small> ETH</small>
      </span>
    );
  }

  render () {
    return (
      <Popup
        wide='very'
        trigger={
          <div className={ [styles.signerPending].join(' ') }>
            <Icon name={ this.store.pending.length > 0 ? 'bell' : 'bell outline' } />
            {this.store.pending.length > 0 &&
              <Label floating color='red' size='mini' circular className={ styles.label }>
                {this.store.pending.length}
              </Label>
            }
          </div>
        }
        content={
          <div>
            <Header
              as='h5'
              icon='lock'
              content={
                <FormattedMessage
                  id='application.status.signerPendingTitle'
                  defaultMessage='Authorization Requests'
                />
              }
            />
            {this.store.pending.length > 0
              ? (
                <List divided relaxed='very'>
                  {this.store.pending.map(request =>
                    <List.Item key={ request.id.toNumber() }>
                      <List.Content floated='right'>
                        <Button
                          icon='unlock alternate'
                          content={
                            <FormattedMessage
                              id='application.status.signerPendingView'
                              defaultMessage='View'
                            />
                          }
                          primary
                          onClick={ this.handleViewRequest }
                          size='mini'
                        />
                      </List.Content>
                      <Image avatar size='mini' verticalAlign='middle'>
                        <IdentityIcon
                          address={ request.payload.sendTransaction.from }
                        />
                      </Image>
                      <List.Content>
                        <List.Header>
                          <FormattedMessage
                            id='application.status.signerPendingSignerRequest'
                            defaultMessage='Parity Signer Request'
                          />
                        </List.Header>
                        <List.Description className={ styles.listDescription }>
                          <FormattedMessage
                            id='application.status.signerPendingSending'
                            defaultMessage='Sending {etherValue} to'
                            values={ { etherValue: this.renderEtherValue(request.payload.sendTransaction.value) } }
                          />
                          <IdentityIcon
                            tiny
                            address={ request.payload.sendTransaction.to }
                            className={ styles.toAvatar }
                          />
                        </List.Description>
                      </List.Content>
                    </List.Item>)
                  }
                </List>
              ) : (
                <Container textAlign='center' fluid className={ styles.noRequest }>
                  <FormattedMessage
                    id='application.status.signerPendingNoRequest'
                    defaultMessage='You have no pending requests.'
                  />
                </Container>
              )}
          </div>
        }
        offset={ 8 } // Empirically looks better
        on='click'
        hideOnScroll
        open={ this.state.isOpen }
        onClose={ this.handleClose }
        onOpen={ this.handleOpen }
        position='bottom right'
      />
    );
  }
}

export default SignerPending;
