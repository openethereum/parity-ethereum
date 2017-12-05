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
import IdentityIcon from '@parity/ui/lib/IdentityIcon';
import Button from 'semantic-ui-react/dist/commonjs/elements/Button';
import Image from 'semantic-ui-react/dist/commonjs/elements/Image';
import List from 'semantic-ui-react/dist/commonjs/elements/List';

import styles from './requestItem.css';

@observer
@connect(({ tokens }, { request: { payload: { sendTransaction } } }) => ({
  token: Object.values(tokens).find(({ address }) => address === sendTransaction.to)
}))
class RequestItem extends Component {
  static propTypes = {
    onClick: PropTypes.func.isRequired,
    request: PropTypes.object.isRequired,
    token: PropTypes.string
  };

  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  state = {
    transaction: null
  };

  methodDecodingStore = MethodDecodingStore.get(this.context.api);

  componentWillMount () {
    const { request: { payload: { sendTransaction } } } = this.props;

    this.methodDecodingStore
      .lookup(sendTransaction.from, sendTransaction)
      .then(lookup => this.setState({
        transaction: lookup
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
    } = this.state.transaction;

    console.log(this.state.transaction);

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
    return (
      <FormattedMessage
        id='application.status.signerPendingContractMethod'
        defaultMessage='Executing method on contract'
      />
    );
  };

  renderTokenTransfer = () => {
    const { request: { payload: { sendTransaction } } } = this.props;

    return (
      <FormattedMessage
        id='application.status.signerendingTokenTransfer'
        defaultMessage='Sending {tokenValue} to'
        values={ {
          tokenValue: this.renderTokenValue(sendTransaction.value)
        }
        }
      />
    );
  };

  renderValueTransfer = () => {
    const { request: { payload: { sendTransaction } } } = this.props;

    return (
      <FormattedMessage
        id='application.status.signerendingValueTransfer'
        defaultMessage='Sending {etherValue} to'
        values={ {
          etherValue: this.renderEtherValue(sendTransaction.value)
        }
        }
      />
    );
  };

  render () {
    const { request: { payload: { sendTransaction } }, onClick } = this.props;

    if (!this.state.transaction) { return null; }

    return (
      <List.Item >
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
            onClick={ onClick }
            size='mini'
          />
        </List.Content>
        <Image avatar size='mini' verticalAlign='middle'>
          <IdentityIcon
            address={ sendTransaction.from }
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
            {this.renderDescription()}
            <IdentityIcon
              tiny
              address={ sendTransaction.to }
              className={ styles.toAvatar }
            />
          </List.Description>
        </List.Content>
      </List.Item >
    );
  }
}

export default RequestItem;
