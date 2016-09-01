
import React, { Component, PropTypes } from 'react';

import { Router, Route, useRouterHistory, IndexRedirect } from 'react-router';
import { createHashHistory } from 'history';
import { syncHistoryWithStore } from 'react-router-redux';

import RootContainer from '../containers/Root';
import LoadingPage from '../containers/LoadingPage';
import RequestsPage from '../containers/RequestsPage';
import UnAuthorizedPage from '../containers/UnAuthorizedPage';
import OfflinePage from '../containers/OfflinePage';

const routerHistory = useRouterHistory(createHashHistory)({});

export default class Routes extends Component {

  render () {
    const { store } = this.props;
    const history = syncHistoryWithStore(routerHistory, store);
    return (
      <Router history={ history }>
        <Route component={ RootContainer }>
          <Route path={ '/loading' } component={ LoadingPage } />
          <Route path={ '/offline' } component={ OfflinePage } />
          <Route path={ '/unAuthorized' } component={ UnAuthorizedPage } />
          <Route path={ '/' } onEnter={ this.requireAuth }>
            <IndexRedirect to='requests' />
            <Route path={ 'requests' } component={ RequestsPage } />
          </Route>
        </Route>
      </Router>
    );
  }

  static propTypes = {
    store: PropTypes.object.isRequired
  };

  requireAuth = (nextState, replace) => {
    const appState = this.props.store.getState().app;
    const { isLoading, isConnected, isNodeRunning } = appState;

    if (isLoading) {
      replace('/loading');
      return;
    }

    if (!isNodeRunning) {
      replace('/offline');
      return;
    }

    if (!isConnected) {
      replace('/unAuthorized');
      return;
    }
  };

}
