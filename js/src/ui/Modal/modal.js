import React, { Component, PropTypes } from 'react';

import { Dialog } from 'material-ui';

import ModalSteps from './ModalSteps';

const ACTIONS_STYLE = { borderStyle: 'none' };
const TITLE_STYLE = { borderStyle: 'none' };
const DIALOG_STYLE = { paddingTop: '1px' };
const CONTENT_STYLE = { transform: 'translate(0px, 0px)' };

export default class Modal extends Component {
  static propTypes = {
    actions: PropTypes.node,
    children: PropTypes.node,
    current: PropTypes.number,
    scroll: PropTypes.bool,
    steps: PropTypes.array,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]),
    visible: PropTypes.bool.isRequired
  }

  render () {
    const title = this.props.steps
      ? (<ModalSteps current={ this.props.current } steps={ this.props.steps } />)
      : this.props.title;

    return (
      <Dialog
        actions={ this.props.actions }
        actionsContainerStyle={ ACTIONS_STYLE }
        autoDetectWindowHeight={ false }
        autoScrollBodyContent={ !!this.props.scroll }
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
