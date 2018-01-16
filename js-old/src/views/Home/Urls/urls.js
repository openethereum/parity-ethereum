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

import { observer } from 'mobx-react';
import moment from 'moment';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';

import { Container, ContainerTitle, DappUrlInput, SectionList } from '~/ui';
import { LinkIcon } from '~/ui/Icons';

import styles from './urls.css';

@observer
export default class Urls extends Component {
  static contextTypes = {
    router: PropTypes.object.isRequired
  };

  static propTypes = {
    extensionStore: PropTypes.object.isRequired,
    store: PropTypes.object.isRequired
  }

  render () {
    const { nextUrl } = this.props.store;

    return (
      <div className={ styles.urls }>
        <div className={ styles.layout }>
          <ContainerTitle
            title={
              <FormattedMessage
                id='home.url.title'
                defaultMessage='Web Applications'
              />
            }
          />
          <DappUrlInput
            className={ styles.input }
            onChange={ this.onChangeUrl }
            onGoto={ this.onGotoUrl }
            onRestore={ this.onRestoreUrl }
            url={ nextUrl }
          />
          { this.renderHistory() }
        </div>
      </div>
    );
  }

  renderHistory () {
    const { history } = this.props.store;

    if (!history.length) {
      return (
        <div className={ styles.empty }>
          <FormattedMessage
            id='home.url.none'
            defaultMessage='No recent URL history available'
          />
        </div>
      );
    }

    return (
      <SectionList
        items={ history }
        renderItem={ this.renderHistoryItem }
      />
    );
  }

  renderHistoryItem = (history) => {
    if (!history || !history.url) {
      return null;
    }

    const onNavigate = () => this.onGotoUrl(history.url);

    return (
      <Container
        className={ styles.historyItem }
        hover={
          <div className={ styles.timestamp }>
            <FormattedMessage
              id='home.url.visited'
              defaultMessage='visited {when}'
              values={ {
                when: moment(history.timestamp).fromNow()
              } }
            />
          </div>
        }
        key={ history.timestamp }
        onClick={ onNavigate }
      >
        <LinkIcon className={ styles.linkIcon } />
        <div className={ styles.url }>
          { history.hostname }
        </div>
      </Container>
    );
  }

  onChangeUrl = (url) => {
    this.props.store.setNextUrl(url);
  }

  onGotoUrl = (url) => {
    const { router } = this.context;
    const { extensionStore } = this.props;

    this.props.store.gotoUrl(url);

    if (extensionStore.hasExtension) {
      window.open(url, '_blank');
    } else {
      router.push('/web');
    }
  }

  onRestoreUrl = () => {
    this.props.store.restoreUrl();
  }
}
