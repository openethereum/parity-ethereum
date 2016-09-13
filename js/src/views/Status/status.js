import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { extend } from 'lodash';

import { Actionbar, Page } from '../../ui';

import { updateLogging } from './actions/logger';
import StatusPage from './containers/StatusPage';

import styles from './status.css';

class Status extends Component {
  static propTypes = {
    status: PropTypes.object.isRequired,
    statusLogger: PropTypes.object.isRequired,
    routing: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired,
    params: PropTypes.object
  }

  render () {
    return (
      <div className={ styles.container }>
        <Actionbar
          title='status' />
        <Page>
          <StatusPage />
        </Page>
      </div>
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators(extend({}, {}, { updateLogging }), dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Status);
