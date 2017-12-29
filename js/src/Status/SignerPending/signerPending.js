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

import Container from 'semantic-ui-react/dist/commonjs/elements/Container';
import Header from 'semantic-ui-react/dist/commonjs/elements/Header';
import Icon from 'semantic-ui-react/dist/commonjs/elements/Icon';
import Label from 'semantic-ui-react/dist/commonjs/elements/Label';
import List from 'semantic-ui-react/dist/commonjs/elements/List';
import Popup from 'semantic-ui-react/dist/commonjs/modules/Popup';

import Store from '../../Signer/pendingStore';
import ParityBarStore from '../../ParityBar/store';
import RequestItem from './RequestItem';
import styles from './signerPending.css';

@observer
class SignerPending extends Component {
  static propTypes = {};

  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  state = {
    isOpen: false
  };

  store = Store.get(this.context.api);
  parityBarStore = ParityBarStore.get();

  handleRequestClick = () => {
    this.parityBarStore.toggleOpenSigner();
    this.handleClose();
  };

  handleOpen = () => {
    this.setState({ isOpen: true });
  };

  handleClose = () => {
    this.setState({ isOpen: false });
  };

  renderPopupContent = () => (
    <div>
      <Header as='h5'>
        <FormattedMessage
          id='application.status.signerPendingTitle'
          defaultMessage='Authorization Requests'
        />
      </Header>
      {this.store.pending.length > 0
        ? (
          <List divided relaxed='very' selection>
            {this.store.pending.map(request => (
              <RequestItem
                transaction={ request.payload.sendTransaction }
                key={ request.id.toNumber() }
                onClick={ this.handleRequestClick }
              />
            ))}
          </List>
        ) : (
          <Container textAlign='center' fluid className={ styles.noRequest }>
            <FormattedMessage
              id='application.status.signerPendingNoRequest'
              defaultMessage='You have no pending requests.'
            />
          </Container>
        )
      }
    </div>
  );

  render () {
    return (
      <Popup
        wide='very'
        trigger={
          <div className={ [styles.signerPending].join(' ') }>
            <Icon
              name={ this.store.pending.length > 0 ? 'bell' : 'bell outline' }
            />
            {this.store.pending.length > 0 && (
              <Label
                floating
                color='red'
                size='mini'
                circular
                className={ styles.label }
              >
                {this.store.pending.length}
              </Label>
            )}
          </div>
        }
        content={ this.renderPopupContent() }
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
