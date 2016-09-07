import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { extend } from 'lodash';

import Header from '../../components/Header';
import Footer from '../../components/Footer';
import * as ToastActions from '../../actions/toastr';
import { updateLogging } from '../../actions/logger';
import ToastrContainer from '../../components/ToastrContainer';
import StatusPage from '../StatusPage';

// TODO [jacogr] get rid of this ASAP
import 'dapp-styles/dist/dapp-styles.css';
import styles from './status.css';

class Container extends Component {
  static propTypes = {
    status: PropTypes.object.isRequired,
    statusLogger: PropTypes.object.isRequired,
    statusToastr: PropTypes.object.isRequired,
    routing: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired
  }

  render () {
    const { actions, statusLogger } = this.props;
    const { name, disconnected, noOfErrors, version } = this.props.status;

    return (
      <div className={ styles.container }>
        <Header
          nodeName={ name }
          disconnected={ disconnected }
          noOfErrors={ noOfErrors }
          { ...this._test('header') }
        />
        { this.renderPage() }
        <Footer
          version={ version }
          logging={ statusLogger.logging }
          updateLogging={ actions.updateLogging }
          { ...this._test('footer') }
        />
        <ToastrContainer { ...this.props } />
      </div>
    );
  }

  renderPage () {
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
