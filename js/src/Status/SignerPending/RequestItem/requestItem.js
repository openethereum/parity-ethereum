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
import { connect } from 'react-redux';
import { observer } from 'mobx-react';
import PropTypes from 'prop-types';
import { FormattedMessage } from 'react-intl';

import MethodDecodingStore from '@parity/ui/lib/MethodDecoding/methodDecodingStore';
import { TOKEN_METHODS } from '@parity/ui/lib/MethodDecoding/constants';
import TokenValue from '@parity/ui/lib/MethodDecoding/tokenValue';
import IdentityIcon from '@parity/ui/lib/IdentityIcon';
import Image from 'semantic-ui-react/dist/commonjs/elements/Image';
import List from 'semantic-ui-react/dist/commonjs/elements/List';

import EtherValue from '../EtherValue';
import styles from './requestItem.css';

@observer
@connect(({ tokens }, { transaction }) => ({
  token: Object.values(tokens).find(({ address }) => address === transaction.to)
}))
class RequestItem extends Component {
  static propTypes = {
    onClick: PropTypes.func.isRequired,
    transaction: PropTypes.object.isRequired,
    token: PropTypes.object
  };

  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  state = {
    decoded: null // Decoded transaction
  };

  methodDecodingStore = MethodDecodingStore.get(this.context.api);

  componentWillMount () {
    const { transaction } = this.props;

    // Decode the transaction and put it into the state
    this.methodDecodingStore
      .lookup(transaction.from, transaction)
      .then(lookup => this.setState({
        decoded: lookup
      }));
  }

  renderDescription = () => {
    // Decide what to display in the description, depending
    // on what type of transaction we're dealing with
    const { token } = this.props;
    const {
      inputs,
      signature,
      contract,
      deploy
    } = this.state.decoded;

    if (deploy) {
      return this.renderDeploy();
    }

    if (contract && signature) {
      if (token && TOKEN_METHODS[signature] && inputs) {
        return this.renderTokenTransfer();
      }
      return this.renderContractMethod();
    }

    return this.renderValueTransfer();
  }

  renderDeploy = () => {
    return (
      <FormattedMessage
        id='application.status.signerPendingContractDeploy'
        defaultMessage='Deploying contract'
      />
    );
  };

  renderContractMethod = () => {
    const { transaction } = this.props;

    return (
      <List.Description className={ styles.listDescription }>
        <FormattedMessage
          id='application.status.signerPendingContractMethod'
          defaultMessage='Executing method on contract'
        />
        {this.renderRecipient(transaction.to)}
      </List.Description>
    );
  };

  renderTokenTransfer = () => {
    const { token } = this.props;
    const { inputs } = this.state.decoded;
    const valueInput = inputs.find(({ name }) => name === '_value');
    const toInput = inputs.find(({ name }) => name === '_to');

    return (
      <List.Description className={ styles.listDescription }>
        <FormattedMessage
          id='application.status.signerendingTokenTransfer'
          defaultMessage='Sending {tokenValue} to'
          values={ {
            tokenValue: (
              <TokenValue value={ valueInput.value } id={ token.id } />
            )
          }
          }
        />
        {this.renderRecipient(toInput.value)}
      </List.Description>
    );
  };

  renderValueTransfer = () => {
    const { transaction } = this.props;

    return (
      <List.Description className={ styles.listDescription }>
        <FormattedMessage
          id='application.status.signerendingValueTransfer'
          defaultMessage='Sending {etherValue} to'
          values={ {
            etherValue: <EtherValue value={ transaction.value } />
          }
          }
        />
        {this.renderRecipient(transaction.to)}
      </List.Description>
    );
  };

  renderRecipient = address => (
    <IdentityIcon
      tiny
      address={ address }
      className={ styles.toAvatar }
    />
  );

  render () {
    const { transaction, onClick } = this.props;

    if (!this.state.decoded) { return null; }

    return (
      <List.Item onClick={ onClick }>
        <Image avatar size='mini' verticalAlign='middle'>
          <IdentityIcon
            className={ styles.fromAvatar }
            address={ transaction.from }
          />
        </Image>
        <List.Content>
          <List.Header>
            <FormattedMessage
              id='application.status.signerPendingSignerRequest'
              defaultMessage='Parity Signer Request'
            />
          </List.Header>
          {this.renderDescription()}
        </List.Content>
      </List.Item >
    );
  }
}

export default RequestItem;
