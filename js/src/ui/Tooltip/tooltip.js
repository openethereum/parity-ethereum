import React, { Component, PropTypes } from 'react';

import { FlatButton } from 'material-ui';
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';
import NavigationArrowForward from 'material-ui/svg-icons/navigation/arrow-forward';

import styles from './style.css';

export default class Tooltip extends Component {
  static contextTypes = {
    tooltips: PropTypes.object
  }

  static propTypes = {
    title: PropTypes.string,
    text: PropTypes.string
  }

  state = {
    tooltipId: 0,
    currentId: 0,
    maxId: 0
  }

  componentDidMount () {
    this.setState({
      tooltipId: this.context.tooltips.register(this.onTooltip)
    });
  }

  render () {
    if (this.state.tooltipId !== this.state.currentId) {
      return null;
    }

    const buttons = this.state.tooltipId !== this.state.maxId
      ? [
        <FlatButton
          key='skipButton'
          icon={ <ContentClear /> }
          label='Skip'
          onTouchTap={ this.onClose } />,
        <FlatButton
          key='nextButton'
          icon={ <NavigationArrowForward /> }
          label='Next'
          onTouchTap={ this.onNext } />
      ] : (
      <FlatButton
        icon={ <ActionDoneAll /> }
        label='Done'
        onTouchTap={ this.onClose } />
      );

    return (
      <div className={ styles.box }>
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

  onNext = () => {
    this.context.tooltips.next();
  }

  onClose = () => {
    this.context.tooltips.close();
  }

  onTooltip = (currentId, maxId) => {
    this.setState({
      currentId: currentId,
      maxId: maxId
    });
  }
}
