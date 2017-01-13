// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import { Link } from 'react-router';
import { isEqual } from 'lodash';
import ReactTooltip from 'react-tooltip';
import { FormattedMessage } from 'react-intl';

import { Balance, Container, ContainerTitle, IdentityIcon, IdentityName, Tags, Input } from '~/ui';
import Certifications from '~/ui/Certifications';
import { nullableProptype } from '~/util/proptypes';

import styles from '../accounts.css';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  };

  static propTypes = {
    account: PropTypes.object.isRequired,
    balance: PropTypes.object,
    link: PropTypes.string,
    name: PropTypes.string,
    noLink: PropTypes.bool,
    showCertifications: PropTypes.bool,
    handleAddSearchToken: PropTypes.func,
    owners: nullableProptype(PropTypes.array)
  };

  static defaultProps = {
    noLink: false,
    showCertifications: false
  };

  shouldComponentUpdate (nextProps) {
    const prev = {
      link: this.props.link, name: this.props.name,
      noLink: this.props.noLink,
      meta: this.props.account.meta, address: this.props.account.address
    };

    const next = {
      link: nextProps.link, name: nextProps.name,
      noLink: nextProps.noLink,
      meta: nextProps.account.meta, address: nextProps.account.address
    };

    if (!isEqual(next, prev)) {
      return true;
    }

    const prevTokens = this.props.balance.tokens || [];
    const nextTokens = nextProps.balance.tokens || [];

    if (prevTokens.length !== nextTokens.length) {
      return true;
    }

    const prevValues = prevTokens.map((t) => ({ value: t.value.toNumber(), image: t.token.image }));
    const nextValues = nextTokens.map((t) => ({ value: t.value.toNumber(), image: t.token.image }));

    if (!isEqual(prevValues, nextValues)) {
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
    const { account, handleAddSearchToken } = this.props;
    const { tags } = account.meta;

    if (!account) {
      return null;
    }

    const { address } = account;

    const addressComponent = (
      <Input
        readOnly
        hideUnderline
        value={ address }
        allowCopy={ address }
      />
    );

    const description = this.getDescription(account.meta);

    return (
      <Container>
        <Tags tags={ tags } handleAddSearchToken={ handleAddSearchToken } />
        <div className={ styles.heading }>
          <IdentityIcon
            address={ address }
          />
          <ContainerTitle
            byline={ addressComponent }
            className={ styles.main }
            description={ description }
            title={ this.renderLink() }
          />
        </div>

        { this.renderOwners() }
        { this.renderBalance() }
        { this.renderCertifications() }
      </Container>
    );
  }

  getDescription (meta = {}) {
    const { blockNumber } = meta;

    if (!blockNumber) {
      return null;
    }

    const formattedBlockNumber = (new BigNumber(blockNumber)).toFormat();

    return (
      <FormattedMessage
        id='accounts.summary.minedBlock'
        defaultMessage='Mined at block #{blockNumber}'
        values={ {
          blockNumber: formattedBlockNumber
        } }
      />
    );
  }

  renderOwners () {
    const { owners } = this.props;
    const ownersValid = (owners || []).filter((owner) => owner.address && new BigNumber(owner.address).gt(0));

    if (!ownersValid || ownersValid.length === 0) {
      return null;
    }

    return (
      <div className={ styles.owners }>
        {
          ownersValid.map((owner, index) => (
            <div key={ `${index}_${owner.address}` }>
              <div
                data-tip
                data-for={ `owner_${owner.address}` }
                data-effect='solid'
              >
                <IdentityIcon address={ owner.address } button />
              </div>
              <ReactTooltip id={ `owner_${owner.address}` }>
                <strong>{ owner.name } </strong><small> (owner)</small>
              </ReactTooltip>
            </div>
          ))
        }
      </div>
    );
  }

  renderLink () {
    const { link, noLink, account, name } = this.props;

    const { address } = account;
    const baseLink = account.wallet
      ? 'wallet'
      : link || 'accounts';

    const viewLink = `/${baseLink}/${address}`;

    const content = (
      <IdentityName address={ address } name={ name } unknown />
    );

    if (noLink) {
      return content;
    }

    return (
      <Link to={ viewLink }>
        { content }
      </Link>
    );
  }

  renderBalance () {
    const { balance } = this.props;

    if (!balance) {
      return null;
    }

    return (
      <Balance balance={ balance } />
    );
  }

  renderCertifications () {
    const { showCertifications, account } = this.props;
    if (!showCertifications) {
      return null;
    }

    return (
      <Certifications address={ account.address } />
    );
  }
}
