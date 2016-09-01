import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { FlatButton } from 'material-ui';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import { newTooltip, nextTooltip, closeTooltips } from '../actions';

import styles from '../style.css';

let tooltipId = 0;

class Tooltip extends Component {
  static propTypes = {
    title: PropTypes.string,
    text: PropTypes.string,
    top: PropTypes.string,
    left: PropTypes.string,
    currentId: PropTypes.number,
    maxId: PropTypes.number,
    onNewTooltip: PropTypes.func,
    onNextTooltip: PropTypes.func,
    onCloseTooltips: PropTypes.func
  }

  state = {
    id: tooltipId
  }

  componentWillMount () {
    const { onNewTooltip } = this.props;

    onNewTooltip(tooltipId);
    tooltipId++;
  }

  render () {
    const { id } = this.state;
    const { currentId, maxId, top, left, onCloseTooltips, onNextTooltip } = this.props;
    const inlineStyles = { top, left };

    if (id !== currentId) {
      return null;
    }

    const buttons = id !== maxId
      ? [
        <FlatButton
          key='skipButton'
          icon={ <ContentClear /> }
          label='Skip'
          onTouchTap={ onCloseTooltips } />,
        <FlatButton
          key='nextButton'
          icon={ <NavigationArrowForward /> }
          label='Next'
          onTouchTap={ onNextTooltip } />
      ] : (
      <FlatButton
        icon={ <ActionDoneAll /> }
        label='Done'
        onTouchTap={ onCloseTooltips } />
      );

    return (
      <div
        style={ inlineStyles }
        className={ styles.box }>
        <div className={ styles.title }>
          { this.props.title }
        </div>
        <div className={ styles.text }>
          { this.props.text }
        </div>
        <div className={ styles.buttons }>
          { buttons }
        </div>
      </div>
    );
  }
}

function mapStateToProps (state) {
  const { currentId, maxId } = state.tooltip;

  return { currentId, maxId };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    onNewTooltip: newTooltip,
    onNextTooltip: nextTooltip,
    onCloseTooltips: closeTooltips
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Tooltip);
