import React, { Component, PropTypes } from 'react';

import { Dialog } from 'material-ui';

import OverlaySteps from './OverlaySteps';

const TITLE_STYLE = { borderStyle: 'none' };
const DIALOG_STYLE = { paddingTop: '1px' };
const CONTENT_STYLE = { transform: 'translate(0px, 0px)' };

export default class Overlay extends Component {
  static propTypes = {
    actions: PropTypes.node,
    children: PropTypes.node,
    current: PropTypes.number,
    steps: PropTypes.array,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]),
    visible: PropTypes.bool.isRequired
  }

  render () {
    const title = this.props.steps
      ? (<OverlaySteps current={ this.props.current } steps={ this.props.steps } />)
      : this.props.title;

    return (
      <Dialog
        actions={ this.props.actions }
        actionsContainerStyle={ TITLE_STYLE }
        autoDetectWindowHeight={ false }
        autoScrollBodyContent={ false }
        contentStyle={ CONTENT_STYLE }
        modal
        open={ this.props.visible }
        repositionOnUpdate={ false }
        style={ DIALOG_STYLE }
        title={ title }
        titleStyle={ TITLE_STYLE }>
        { this.props.children }
      </Dialog>
    );
  }
}
