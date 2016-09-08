import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { extend } from 'lodash';

import Header from '../../components/Header';
import * as ToastActions from '../../actions/toastr';
import { updateLogging } from '../../actions/logger';
import ToastrContainer from '../../components/ToastrContainer';

import DebugPage from '../DebugPage';
import RpcPage from '../RpcPage';
import StatusPage from '../StatusPage';

import styles from './status.css';

class Container extends Component {
  static propTypes = {
    status: PropTypes.object.isRequired,
    statusLogger: PropTypes.object.isRequired,
    statusToastr: PropTypes.object.isRequired,
    routing: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired,
    params: PropTypes.object
  }

  render () {
    const { name, disconnected, noOfErrors } = this.props.status;

    return (
      <div className={ styles.container }>
        <Header
          nodeName={ name }
          disconnected={ disconnected }
          noOfErrors={ noOfErrors }
          { ...this._test('header') }
        />
        { this.renderPage() }
        <ToastrContainer { ...this.props } />
      </div>
    );
  }

  renderPage () {
    const { params } = this.props;

    if (params && params.subpage) {
      if (params.subpage === 'debug') {
        return (
          <DebugPage />
        );
      } else if (params.subpage === 'rpc') {
        return (
          <RpcPage>
            <div>This is very much still a WIP, hence the original RPC calls are not available here yet (it should actually be removed here and moved to a dedicated developer section once available)</div>
          </RpcPage>
        );
      }
    }

    return (
      <StatusPage />
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators(extend({}, ToastActions, { updateLogging }), dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Container);
