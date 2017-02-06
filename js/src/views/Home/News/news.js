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

import React, { Component } from 'react';
import ReactMarkdown from 'react-markdown';

import Contracts from '~/contracts';
import { SectionList } from '~/ui';

import { createRenderers } from './renderers';
import styles from './news.css';

const VERSION_ID = '1';

export default class News extends Component {
  componentWillMount () {
    return this.retrieveNews();
  }

  state = {
    newsItems: null
  }

  render () {
    const { newsItems } = this.state;

    if (!newsItems || !newsItems.length) {
      return null;
    }

    return (
      <SectionList
        className={ styles.news }
        items={ newsItems }
        renderItem={ this.renderItem }
      />
    );
  }

  renderItem = (item) => {
    if (!item) {
      return null;
    }

    const inlineStyles = {
      body: item.style ? (item.style.body || {}) : {},
      head: item.style ? (item.style.head || {}) : {},
      tags: item.style ? (item.style.tags || {}) : {}
    };

    return (
      <div className={ styles.item }>
        <div
          className={ styles.background }
          style={ {
            backgroundImage: `url(${item.background})`
          } }
        />
        <div
          className={ styles.title }
          style={ inlineStyles.head }
        >
          { item.title }
        </div>
        <div
          className={ styles.overlay }
          style={ inlineStyles.body }
        >
          <ReactMarkdown
            className={ styles.markdown }
            renderers={ createRenderers(inlineStyles.tags) }
            source={ item.markdown }
            softBreak='br'
          />
        </div>
      </div>
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

        // FIXME: just for testing, not owner of paritynews
        url = 'https://raw.githubusercontent.com/jacogr/parity-news/cab9e2814d84e0c8b3215406d38242bebbe531ea/news.json';
        return fetch(url).then((response) => {
          if (!response.ok) {
            return null;
          }

          return response.json();
        });
      })
      .then((news) => {
        if (news && news[VERSION_ID]) {
          this.setState({ newsItems: news[VERSION_ID].items });
        }
      });
  }
}

export {
  VERSION_ID
};
