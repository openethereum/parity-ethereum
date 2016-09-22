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
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { RaisedButton } from 'material-ui';

import { Container, ContainerTitle, Input } from '../../../../ui';
import { updateToken } from '../../actions/signer';

import styles from './UnAuthorizedPage.css';

class UnAuthorizedPage extends Component {
  static propTypes = {
    signer: PropTypes.shape({
      token: PropTypes.string.isRequired
    }).isRequired,
    actions: PropTypes.shape({
      updateToken: PropTypes.func.isRequired
    }).isRequired
  }

  state = {
    token: this.props.signer.token,
    tokenInvalid: null,
    processing: false
  };

  componentWillReceiveProps (nextProps) {
    if (this.props.signer.token === nextProps.signer.token) {
      return;
    }
    this.setState({
      token: nextProps.signer.token
    });
  }

  componentWillUnmount () {
    clearTimeout(this.tokenInvalidTimeout);
  }

  render () {
    const { processing, token } = this.state;
    return (
      <Container>
        <ContainerTitle title='Not Authorized' />
        <div className={ styles.section }>
          Connections used by Trusted Signer are secured. You need to authorize this application.
        </div>
        <div className={ styles.section }>
          Make sure Parity is running and generate an authorization token with <code className={ styles.code }>parity signer new-token</code> in your console, pasting the token below:
        </div>
        <div className={ styles.section }>
          <Input
            value={ token }
            disabled={ processing }
            onChange={ this.onTokenChange }
            hint='token from Parity'
            label='Authorization Token' />
          <br />
          <RaisedButton
            primary
            onClick={ this.onSubmit }
            disabled={ processing || !token }
            label='Authorize'
           />
          { this.renderInvalidToken() }
          { this.renderProcessing() }
        </div>
      </Container>
    );
  }

  onTokenChange = evt => {
    this.setState({ token: evt.target.value, tokenInvalid: false });
  }

  onSubmit = () => {
    const token = this.state.token.replace(/[^a-zA-Z0-9]/g, '');
    this.setState({
      processing: true,
      tokenInvalid: false
    });
    this.props.actions.updateToken(token);

    // todo [adgo] - listen to event instead of timeout
    this.tokenInvalidTimeout = setTimeout(this.onTokenInvalid, 4000); // if token is valid this component should unmount. after 4 sconds we assume it's invalid.
  }

  renderProcessing () {
    if (!this.state.processing) {
      return null;
    }

    return (
      <span> Processing ...</span>
    );
  }

  renderInvalidToken () {
    if (!this.state.tokenInvalid) {
      return null;
    }

    return <span> The token is invalid or your node is not running.</span>;
  }

  onTokenInvalid = () => {
    this.setState({
      processing: false,
      tokenInvalid: true
    });
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators({ updateToken }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(UnAuthorizedPage);
