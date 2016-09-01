import React, { Component, PropTypes } from 'react';
import { Route, IndexRedirect } from 'react-router';

import RootContainer from '../containers/Root';
import LoadingPage from '../containers/LoadingPage';
import RequestsPage from '../containers/RequestsPage';
import UnAuthorizedPage from '../containers/UnAuthorizedPage';
import OfflinePage from '../containers/OfflinePage';

export default class Routes extends Component {
  static contextTypes = {
    store: PropTypes.object
  }

  static propTypes = {
    path: PropTypes.string
  };

  render () {
    const { path } = this.props;
    console.log('path', path);

    return (
      <Route path={ path || '/' } component={ RootContainer }>
        <Route path={ 'loading' } component={ LoadingPage } />
        <Route path={ 'offline' } component={ OfflinePage } />
        <Route path={ 'unAuthorized' } component={ UnAuthorizedPage } />
        <Route path={ '/' } onEnter={ this.requireAuth }>
          <IndexRedirect to='requests' />
          <Route path={ 'requests' } component={ RequestsPage } />
        </Route>
      </Route>
    );
  }

  requireAuth = (nextState, replace) => {
    const { store } = this.context;
    const appState = store.getState().app;
    const { isLoading, isConnected, isNodeRunning } = appState;

    if (isLoading) {
      replace('loading');
    } else if (!isNodeRunning) {
      replace('offline');
    } else if (!isConnected) {
      replace('unAuthorized');
    }
  };
}
