import React, { Component, PropTypes } from 'react';
import { bindActionCreators } from 'redux';
import { connect } from 'react-redux';

import { clearStatusLogs, toggleStatusLogs } from '../../../../redux/actions';

import Debug from '../../components/Debug';
import Status from '../../components/Status';

import styles from './statusPage.css';

class StatusPage extends Component {
  static propTypes = {
    nodeStatus: PropTypes.object.isRequired,
    actions: PropTypes.object.isRequired
  }

  render () {
    return (
      <div className={ styles.body }>
        <Status { ...this.props } />
        <Debug { ...this.props } />
      </div>
    );
  }
}

function mapStateToProps (state) {
  return state;
}

function mapDispatchToProps (dispatch) {
  return {
    actions: bindActionCreators({
      clearStatusLogs,
      toggleStatusLogs
    }, dispatch)
  };
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(StatusPage);
