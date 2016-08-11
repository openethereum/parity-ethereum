import React, { Component, PropTypes } from 'react';

import getMuiTheme from 'material-ui/styles/getMuiTheme';
import darkBaseTheme from 'material-ui/styles/baseThemes/darkBaseTheme';
import lightBaseTheme from 'material-ui/styles/baseThemes/lightBaseTheme';

import Api from '../../api';

import FirstRun from '../FirstRun';
import Status from './Status';
import TabBar from './TabBar';

import styles from './style.css';

const lightTheme = getMuiTheme(lightBaseTheme);
const muiTheme = getMuiTheme(darkBaseTheme);
const api = new Api(new Api.Transport.Http('/rpc/'));

muiTheme.stepper.textColor = '#eee';
muiTheme.stepper.disabledTextColor = '#777';
muiTheme.inkBar.backgroundColor = 'rgb(0, 151, 167)';
muiTheme.tabs = lightTheme.tabs;
muiTheme.tabs.backgroundColor = 'rgb(65, 65, 65)';
muiTheme.toolbar = lightTheme.toolbar;
muiTheme.toolbar.backgroundColor = 'rgb(80, 80, 80)';

export default class Application extends Component {
  static childContextTypes = {
    api: PropTypes.object,
    muiTheme: PropTypes.object
  }

  static propTypes = {
    children: PropTypes.node
  }

  state = {
    showFirst: false,
    accounts: []
  }

  componentWillMount () {
    this.retrieveAccounts();
  }

  render () {
    return (
      <div className={ styles.container }>
        <FirstRun
          onClose={ this.onCloseFirst }
          visible={ this.state.showFirst } />
        <TabBar />
        { this.props.children }
        <Status />
      </div>
    );
  }

  getChildContext () {
    return {
      api: api,
      muiTheme: muiTheme
    };
  }

  retrieveAccounts () {
    api.personal
      .listAccounts()
      .then((accounts) => {
        this.setState({
          accounts: accounts,
          showFirst: accounts.length === 0
        });
      });
  }

  onCloseFirst = () => {
    this.setState({
      showFirst: false
    });
  }
}
