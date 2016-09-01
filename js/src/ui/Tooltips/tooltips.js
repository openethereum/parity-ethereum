import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { nextTooltip } from './actions';
import { LS_KEY } from './reducers';

import styles from './style.css';

class Tooltips extends Component {
  static contextTypes = {
    store: PropTypes.object
  }

  static propTypes = {
    currentId: PropTypes.number,
    closed: PropTypes.bool,
    onNextTooltip: PropTypes.func
  }

  componentDidMount () {
    const hideTips = false && window.localStorage.getItem(LS_KEY);

    if (!hideTips) {
      this.props.onNextTooltip();
    }
  }

  render () {
    const { currentId } = this.props;

    if (currentId === -1) {
      return null;
    }

    return (
      <div className={ styles.overlay } />
    );
  }
}

function mapStateToProps (state) {
  const { currentId } = state.tooltipReducer;

  return { currentId };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onNextTooltip: nextTooltip
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Tooltips);
