
import React, { Component, PropTypes } from 'react';

import { Router, Route, useRouterHistory, IndexRedirect } from 'react-router';
import { createHashHistory } from 'history';
import { syncHistoryWithStore } from 'react-router-redux';

import AppContainer from './containers/App';
import StatusPage from './containers/StatusPage';
import DebugPage from './containers/DebugPage';
import AccountsPage from './containers/AccountsPage';
import RpcPage from './containers/RpcPage';
import RpcCalls from './components/RpcCalls';
import RpcDocs from './components/RpcDocs';

const routerHistory = useRouterHistory(createHashHistory)({
  queryKey: false
});

export default class Routes extends Component {

  render () {
    const history = syncHistoryWithStore(routerHistory, this.props.store);
    return (
      <Router history={ history }>
        <Route path={ '/' } component={ AppContainer }>
          <IndexRedirect to='status' />
          <Route path={ 'status' } component={ StatusPage } />
          <Route path={ 'debug' } component={ DebugPage } />
          <Route path={ 'accounts' } component={ AccountsPage } />
          <Route path={ 'rpc' } component={ RpcPage }>
            <IndexRedirect to='calls' />
            <Route path={ 'calls' } component={ RpcCalls } />
            <Route path={ 'docs' } component={ RpcDocs } />
          </Route>
        </Route>
      </Router>
    );
  }

  static propTypes = {
    store: PropTypes.object.isRequired
  }

}
