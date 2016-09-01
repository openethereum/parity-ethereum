
import React, { Component, PropTypes } from 'react';

import { Router, Route, IndexRedirect, hashHistory } from 'react-router';
import { syncHistoryWithStore } from 'react-router-redux';

import RootContainer from '../containers/Root';
import WelcomePage from '../containers/WelcomePage';
import AccountPage from '../containers/AccountPage';
import AccountLinkPage from '../containers/AccountLinkPage';
import IdenticonPage from '../containers/IdenticonPage';
import RpcAutoCompletePage from '../containers/RpcAutoCompletePage';
import ToastPage from '../containers/ToastPage';
import TransactionFinishedPage from '../containers/TransactionFinishedPage';
import TransactionPendingPage from '../containers/TransactionPendingPage';
import TransactionPendingWeb3Page from '../containers/TransactionPendingWeb3Page';
import TransactionPendingFormPage from '../containers/TransactionPendingFormPage';

export default class Routes extends Component {

  static propTypes = {
    store: PropTypes.object.isRequired
  };

  render () {
    const { store } = this.props;
    const history = syncHistoryWithStore(hashHistory, store);
    return (
      <Router history={ history }>
        <Route path={ '/' } component={ RootContainer }>
          <IndexRedirect to='welcome' />
          <Route path={ 'welcome' } component={ WelcomePage } />
          <Route path={ 'account' } component={ AccountPage } />
          <Route path={ 'AccountLink' } component={ AccountLinkPage } />
          <Route path={ 'identicon' } component={ IdenticonPage } />
          <Route path={ 'rpcAutoComplete' } component={ RpcAutoCompletePage } />
          <Route path={ 'toast' } component={ ToastPage } />
          <Route path={ 'transactionFinished' } component={ TransactionFinishedPage } />
          <Route path={ 'transactionPending' } component={ TransactionPendingPage } />
          <Route path={ 'transactionPendingWeb3' } component={ TransactionPendingWeb3Page } />
          <Route path={ 'transactionPendingForm' } component={ TransactionPendingFormPage } />
        </Route>
      </Router>
    );
  }
}
