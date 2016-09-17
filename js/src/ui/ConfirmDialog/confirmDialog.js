import React, { Component, PropTypes } from 'react';
import ActionDone from 'material-ui/svg-icons/action/done';
import ContentClear from 'material-ui/svg-icons/content/clear';

import Button from '../Button';
import Modal from '../Modal';

import styles from './confirmDialog.css';

export default class ConfirmDialog extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired,
    iconNo: PropTypes.node,
    iconYes: PropTypes.node,
    labelNo: PropTypes.string,
    labelYes: PropTypes.string,
    title: PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]).isRequired,
    visible: PropTypes.bool.isRequired,
    onNo: PropTypes.func.isRequired,
    onYes: PropTypes.func.isRequired
  }

  render () {
    const { children, title, visible } = this.props;

    return (
      <Modal
        actions={ this.renderActions() }
        title={ title }
        visible={ visible }>
        <div className={ styles.body }>
          { children }
        </div>
      </Modal>
    );
  }

  renderActions () {
    const { iconNo, iconYes, labelNo, labelYes, onNo, onYes } = this.props;

    return [
      <Button
        label={ labelNo || 'no' }
        icon={ iconNo || <ContentClear /> }
        onClick={ onNo } />,
      <Button
        label={ labelYes || 'yes' }
        icon={ iconYes || <ActionDone /> }
        onClick={ onYes } />
    ];
  }
}
