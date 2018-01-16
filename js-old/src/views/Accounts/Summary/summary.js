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

import BigNumber from 'bignumber.js';
import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { Link } from 'react-router';
import { isEqual } from 'lodash';
import ReactTooltip from 'react-tooltip';
import { FormattedMessage } from 'react-intl';

import { Balance, Container, ContainerTitle, CopyToClipboard, IdentityIcon, IdentityName, Tags, VaultTag } from '~/ui';
import Certifications from '~/ui/Certifications';
import { arrayOrObjectProptype, nullableProptype } from '~/util/proptypes';

import styles from '../accounts.css';

class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  };

  static propTypes = {
    account: PropTypes.object.isRequired,
    accountsInfo: PropTypes.object.isRequired,
    disabled: PropTypes.bool,
    link: PropTypes.string,
    name: PropTypes.string,
    noLink: PropTypes.bool,
    showCertifications: PropTypes.bool,
    handleAddSearchToken: PropTypes.func,
    owners: nullableProptype(arrayOrObjectProptype())
  };

  static defaultProps = {
    noLink: false,
    showCertifications: false
  };

  shouldComponentUpdate (nextProps) {
    const prev = {
      link: this.props.link,
      disabled: this.props.disabled,
      name: this.props.name,
      noLink: this.props.noLink,
      meta: this.props.account.meta,
      address: this.props.account.address
    };

    const next = {
      link: nextProps.link,
      disabled: nextProps.disabled,
      name: nextProps.name,
      noLink: nextProps.noLink,
      meta: nextProps.account.meta,
      address: nextProps.account.address
    };

    if (!isEqual(next, prev)) {
      return true;
    }

    const prevOwners = this.props.owners;
    const nextOwners = nextProps.owners;

    if (!isEqual(prevOwners, nextOwners)) {
      return true;
    }

    return false;
  }

  render () {
    const { account, disabled, handleAddSearchToken, noLink } = this.props;
    const { tags } = account.meta;

    if (!account) {
      return null;
    }

    const { address } = account;

    return (
      <Container
        className={ styles.account }
        hover={
          <div className={ styles.overlay }>
            { this.renderBalance(false) }
            { this.renderDescription(account.meta) }
            { this.renderOwners() }
            { this.renderVault(account.meta) }
          </div>
        }
        link={ this.getLink() }
      >
        <Tags
          className={ styles.tags }
          tags={ tags }
          handleAddSearchToken={ handleAddSearchToken }
        />
        <div className={ styles.heading }>
          <IdentityIcon
            address={ address }
            disabled={ disabled }
          />
          <ContainerTitle
            byline={
              <div className={ styles.addressline }>
                <CopyToClipboard data={ address } />
                <div className={ styles.address }>{ address }</div>
              </div>
            }
            className={
              noLink
                ? styles.main
                : styles.mainLink
            }
            title={
              <IdentityName
                address={ address }
                name={ name }
                unknown
              />
            }
          />
        </div>
        <div className={ styles.summary }>
          { this.renderBalance(true) }
        </div>
        { this.renderCertifications(true) }
      </Container>
    );
  }

  renderDescription (meta = {}) {
    const { blockNumber } = meta;

    if (!blockNumber) {
      return null;
    }

    return (
      <div className={ styles.blockDescription }>
        <FormattedMessage
          id='accounts.summary.minedBlock'
          defaultMessage='Mined at block #{blockNumber}'
          values={ {
            blockNumber: (new BigNumber(blockNumber)).toFormat()
          } }
        />
      </div>
    );
  }

  renderOwners () {
    const { accountsInfo, owners } = this.props;
    const ownersValid = (owners || []).filter((owner) => owner.address && new BigNumber(owner.address).gt(0));

    if (!ownersValid || ownersValid.length === 0) {
      return null;
    }

    return (
      <div className={ styles.owners }>
        {
          ownersValid.map((owner, index) => {
            const account = accountsInfo[owner.address];
            let ownerLinkType = 'addresses';

            if (account) {
              if (account.uuid || account.hardware) {
                ownerLinkType = 'accounts';
              } else if (account.wallet) {
                ownerLinkType = 'wallet';
              } else if (account.meta.contract) {
                ownerLinkType = 'contract';
              }
            }

            return (
              <Link
                className={ styles.owner }
                key={ `${index}_${owner.address}` }
                to={ `/${ownerLinkType}/${owner.address}` }
              >
                <div
                  data-tip
                  data-for={ `owner_${owner.address}` }
                  data-effect='solid'
                >
                  <IdentityIcon
                    address={ owner.address }
                    center
                  />
                </div>
                <ReactTooltip id={ `owner_${owner.address}` }>
                  <FormattedMessage
                    id='accounts.tooltips.owner'
                    defaultMessage='{name} (owner)'
                    values={ {
                      name: owner.name
                    } }
                  />
                </ReactTooltip>
              </Link>
            );
          })
        }
      </div>
    );
  }

  getLink () {
    const { link, account } = this.props;
    const { address } = account;
    const baseLink = account.wallet
      ? 'wallet'
      : link || 'accounts';

    return `/${baseLink}/${address}`;
  }

  renderBalance (onlyEth) {
    const { account } = this.props;

    return (
      <Balance
        address={ account.address }
        className={
          onlyEth
            ? styles.ethBalances
            : styles.allBalances
        }
        showOnlyEth={ onlyEth }
      />
    );
  }

  renderCertifications (onlyIcon) {
    const { showCertifications, account } = this.props;

    if (!showCertifications) {
      return null;
    }

    return (
      <Certifications
        address={ account.address }
        className={
          onlyIcon
            ? styles.iconCertifications
            : styles.fullCertifications
        }
        showOnlyIcon={ onlyIcon }
      />
    );
  }

  renderVault (meta) {
    if (!meta || !meta.vault) {
      return null;
    }

    return (
      <VaultTag vault={ meta.vault } />
    );
  }
}

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;

  return {
    accountsInfo
  };
}

export default connect(
  mapStateToProps,
  null
)(Summary);
