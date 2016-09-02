import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { RaisedButton, TextField } from 'material-ui';

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
      <div className={ styles.container }>
        <h2>Not Authorized</h2>
        <p className={ styles.lead }>
          Connections used by Trusted Signer are secured. You need to authorize this app.
        </p>
        <p>Make sure Parity is running, generate authorization token:</p>
        <p><code className={ styles.code }>$ parity signer new-token</code> &nbsp;and paste the token below:</p>
        <TextField
          name='token'
          type='text'
          value={ token }
          disabled={ processing }
          onChange={ this.onTokenChange }
          floatingLabelText='Authorization Token'
        />
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
