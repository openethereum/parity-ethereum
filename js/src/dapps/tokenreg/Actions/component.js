import React, { Component, PropTypes } from 'react';

import { RaisedButton } from 'material-ui';
import ContentSendIcon from 'material-ui/svg-icons/content/send';

import Register from './Register';

import styles from './actions.css';

const REGISTER = 'REGISTER';

export default class Actions extends Component {

  static propTypes = {
    handleRegisterToken: PropTypes.func,
    handleRegisterClose: PropTypes.func,

    register: PropTypes.object
  };

  state = {
    show: {
      [ REGISTER ]: false
    }
  }

  constructor () {
    super();

    this.onRegisterClose = this.onRegisterClose.bind(this);
    this.onShow = this.onShow.bind(this, REGISTER);
  }

  render () {
    return (
      <div className={ styles.actions }>
        <RaisedButton
          className={ styles.button }
          icon={ <ContentSendIcon /> }
          label='Register Token'
          primary
          onTouchTap={ this.onShow } />

        <Register
          show={ this.state.show[ REGISTER ] }
          onClose={ this.onRegisterClose }
          handleRegisterToken={ this.props.handleRegisterToken }
          { ...this.props.register } />
      </div>
    );
  }

  onRegisterClose () {
    this.onHide(REGISTER);
    this.props.handleRegisterClose();
  }

  onShow (key) {
    this.setState({
      show: {
        [ key ]: true
      }
    });
  }

  onHide (key) {
    this.setState({
      show: {
        [ key ]: false
      }
    });
  }

}
