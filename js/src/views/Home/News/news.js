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

import React, { Component } from 'react';
import { FormattedMessage } from 'react-intl';
import ReactMarkdown from 'react-markdown';

import Contracts from '~/contracts';
import { Container } from '~/ui';

import styles from '../home.css';

export default class News extends Component {
  componentWillMount () {
    return this.retrieveNews();
  }

  state = {
    news: null
  }

  render () {
    const { news } = this.state;

    if (!news) {
      return null;
    }

    return (
      <Container
        title={
          <FormattedMessage
            id='home.news.title'
            defaultMessage='Parity News'
          />
        }
      >
        <div className={ styles.news }>
          <ReactMarkdown source={ news } />
        </div>
      </Container>
    );
  }

  retrieveNews () {
    const contracts = Contracts.get();

    return contracts.registry
      .lookupMeta('paritynews', 'CONTENT')
      .then((contentId) => {
        return contracts.githubHint.getEntry(contentId);
      })
      .then(([url, owner, commit]) => {
        if (!url) {
          return null;
        }

        return fetch(url).then((response) => {
          if (!response.ok) {
            console.warn('Unable to retrieve lastest Parity news');
            return null;
          }

          return response.text();
        });
      })
      .then((news) => {
        this.setState({ news });
      });
  }
}
