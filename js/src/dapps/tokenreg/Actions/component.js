import React, { Component, PropTypes } from 'react';

import { RaisedButton } from 'material-ui';
import ActionSearchIcon from 'material-ui/svg-icons/action/search';
import ContentSendIcon from 'material-ui/svg-icons/content/send';

import Register from './Register';
import Query from './Query';

import styles from './actions.css';

const REGISTER_ACTION = 'REGISTER_ACTION';
const QUERY_ACTION = 'QUERY_ACTION';

export default class Actions extends Component {

  static propTypes = {
    handleRegisterToken: PropTypes.func,
    handleRegisterClose: PropTypes.func,
    register: PropTypes.object,

    handleQueryToken: PropTypes.func,
    handleQueryClose: PropTypes.func,
    query: PropTypes.object
  };

  state = {
    show: {
      [ REGISTER_ACTION ]: false,
      [ QUERY_ACTION ]: false
    }
  }

  constructor () {
    super();

    this.onShowRegister = this.onShow.bind(this, REGISTER_ACTION);
    this.onShowQuery = this.onShow.bind(this, QUERY_ACTION);
  }

  render () {
    return (
      <div className={ styles.actions }>
        <RaisedButton
          className={ styles.button }
          icon={ <ContentSendIcon /> }
          label='Register Token'
          primary
          onTouchTap={ this.onShowRegister } />

        <RaisedButton
          className={ styles.button }
          icon={ <ActionSearchIcon /> }
          label='Search Token'
          primary
          onTouchTap={ this.onShowQuery } />

        <Register
          show={ this.state.show[ REGISTER_ACTION ] }
          onClose={ this.onRegisterClose }
          handleRegisterToken={ this.props.handleRegisterToken }
          { ...this.props.register } />

        <Query
          show={ this.state.show[ QUERY_ACTION ] }
          onClose={ this.onQueryClose }
          handleQueryToken={ this.props.handleQueryToken }
          { ...this.props.query } />
      </div>
    );
  }

  onRegisterClose = () => {
    this.onHide(REGISTER_ACTION);
    this.props.handleRegisterClose();
  }

  onQueryClose = () => {
    this.onHide(QUERY_ACTION);
    this.props.handleQueryClose();
  }

  onShow (key) {
    this.setState({
      show: {
        ...this.state.show,
        [ key ]: true
      }
    });
  }

  onHide (key) {
    this.setState({
      show: {
        ...this.state.show,
        [ key ]: false
      }
    });
  }

}
