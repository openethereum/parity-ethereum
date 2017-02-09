// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { nodeOrStringProptype } from '~/util/proptypes';

import Button from '../Button';
import Portal from '../Portal';
import { CancelIcon, CheckIcon } from '../Icons';

import styles from './confirmDialog.css';

const DEFAULT_NO = (
  <FormattedMessage
    id='ui.confirmDialog.no'
    defaultMessage='no'
  />
);
const DEFAULT_YES = (
  <FormattedMessage
    id='ui.confirmDialog.yes'
    defaultMessage='yes'
  />
);

export default class ConfirmDialog extends Component {
  static propTypes = {
    children: PropTypes.node.isRequired,
    className: PropTypes.string,
    disabledConfirm: PropTypes.bool,
    disabledDeny: PropTypes.bool,
    busy: PropTypes.bool,
    iconConfirm: PropTypes.node,
    iconDeny: PropTypes.node,
    labelConfirm: PropTypes.string,
    labelDeny: PropTypes.string,
    onConfirm: PropTypes.func.isRequired,
    onDeny: PropTypes.func.isRequired,
    open: PropTypes.bool,
    title: nodeOrStringProptype().isRequired,
    visible: PropTypes.bool
  }

  render () {
    const { busy, children, className, disabledConfirm, disabledDeny, iconConfirm, iconDeny, labelConfirm, labelDeny, onConfirm, onDeny, open, title, visible } = this.props;

    // TODO: visible is for compatibility with existing, open aligns with Portal.
    // (Cleanup once all uses of ConfirmDialog has been migrated)
    if (!visible && !open) {
      return null;
    }

    return (
      <Portal
        buttons={ [
          <Button
            disabled={ disabledDeny }
            icon={ iconDeny || <CancelIcon /> }
            key='deny'
            label={ labelDeny || DEFAULT_NO }
            onClick={ onDeny }
          />,
          <Button
            disabled={ disabledConfirm }
            icon={ iconConfirm || <CheckIcon /> }
            key='confirm'
            label={ labelConfirm || DEFAULT_YES }
            onClick={ onConfirm }
          />
        ] }
        busy={ busy }
        className={ className }
        isSmallModal
        onClose={ onDeny }
        title={ title }
        open
      >
        <div className={ styles.body }>
          { children }
        </div>
      </Portal>
    );
  }
}
