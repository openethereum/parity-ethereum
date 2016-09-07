import React, { Component, PropTypes } from 'react';
import { keccak_256 } from 'js-sha3'; // eslint-disable-line camelcase
import ActionFingerprint from 'material-ui/svg-icons/action/fingerprint';

import IdentityIcon from '../IdentityIcon';

export default class SignerIcon extends Component {
  static propTypes = {
    className: PropTypes.string
  }

  render () {
    const { className } = this.props;
    const signerToken = window.localStorage.getItem('sysuiToken');

    if (!signerToken) {
      return (
        <ActionFingerprint />
      );
    }

    const signerSha = keccak_256(signerToken);

    return (
      <IdentityIcon
        center
        className={ className }
        address={ signerSha } />
    );
  }
}
