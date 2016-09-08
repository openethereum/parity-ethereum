import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';
import { extend } from 'lodash';

import Status from '../../components/Status';
import * as ModifyMiningActions from '../../actions/modify-mining';
import { updateLogging } from '../../actions/logger';

import styles from './statusPage.css';

class StatusPage extends Component {
  static propTypes = {
    status: PropTypes.object.isRequired,
    statusSettings: PropTypes.object.isRequired,
    statusMining: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired
  }

  render () {
    return (
      <div className={ styles.body }>
        <Status { ...this.props } />
      </div>
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators(extend({}, ModifyMiningActions, { updateLogging }), dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(StatusPage);
