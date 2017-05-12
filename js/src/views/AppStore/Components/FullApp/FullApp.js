/* @flow */
import React, { Component } from 'react';

/** Components **/
import Switch from '../UI/Switch/Switch';

/** Stylesheets **/
import './FullApp.css';

/** Assets **/
import Photo1 from '../../Assets/1.jpg';
import Photo2 from '../../Assets/2.jpg';
import Photo3 from '../../Assets/3.jpg';
import Photo4 from '../../Assets/4.jpg';
import Photo5 from '../../Assets/5.jpg';

// type Props = {|
//   appId: string
// |}
//
// type State = {|
//   readMoreLessText:  string | null,
//   readMoreLessStyle: Object | null,
//   photo: string
// |}

const contentExample = `- Design 2017. Get this App now! -\n This is line wrap test \n\n Some more text... \n\n - One\n - Two\n - Three\n - Four \n\n Lots of text now, Sed rutrum, diam ac accumsan suscipit, lectus sem aliquet quam, nec scelerisque ligula arcu eu mi. Cras eget orci feugiat, sagittis purus vitae, ultricies purus. Integer aliquam vehicula dolor, nec malesuada urna aliquam ut. Vivamus quis tellus quis purus imperdiet lobortis vitae id lorem. Etiam cursus metus est, at suscipit dolor facilisis ac. Suspendisse pharetra rutrum massa et vehicula. Nullam vel sapien purus.`;

const miniApp = (
  <div className="mini-app">
    <svg x="0px" y="0px" width="512px" height="512px" viewBox="0 0 512 512">
      <linearGradient id="SVGID_1_" gradientUnits="userSpaceOnUse" x1="3.0005" y1="509" x2="508.9999" y2="3.0005">
        <stop offset="0" style={{stopColor:"#48E0CA"}}/>
        <stop offset="0.25" style={{stopColor:"#026BDF"}}/>
        <stop offset="0.5" style={{stopColor:"#605BB1"}}/>
        <stop offset="0.75" style={{stopColor:"#EF4C65"}}/>
        <stop offset="1" style={{stopColor:"#E01583"}}/>
      </linearGradient>
      <path fill="none" stroke="url(#SVGID_1_)" strokeWidth="2.5" strokeMiterlimit="10" d="M3,3l506,506 M3,509L509,3 M35.527,3v506
       M477.394,3v506 M3,34.752h506 M3,475.477h506 M509,117.48C509,54.256,457.749,3,394.522,3H117.481C54.256,3,3,54.256,3,117.48
      V394.52C3,457.746,54.256,509,117.481,509h277.042C457.749,509,509,457.746,509,394.52V117.48z M349.182,256
      c0-51.463-41.716-93.186-93.18-93.186c-51.464,0-93.185,41.723-93.185,93.186c0,51.464,41.721,93.184,93.185,93.184
      C307.466,349.184,349.182,307.464,349.182,256z M388.109,256c0-72.96-59.149-132.107-132.107-132.107
      c-72.96,0-132.106,59.146-132.106,132.107c0,72.961,59.146,132.104,132.106,132.104C328.96,388.104,388.109,328.961,388.109,256z
       M455.568,256c0-110.22-89.348-199.57-199.566-199.57C145.783,56.43,56.431,145.78,56.431,256
      c0,110.223,89.352,199.565,199.571,199.565C366.221,455.565,455.568,366.223,455.568,256z M162.817,3v506 M256.002,3v506 M349.182,3
      v506 M3,162.814h506 M3,256h506 M3,349.184h506"/>
    </svg>
    <div className="mini-app-name">App Name Goes Here</div>
    <div className="mini-app-catagory">Catagory</div>
  </div>
);

class FullApp extends Component {
  // props: Props;
  // state: State = {
  //   readMoreLessText: '...Read more',
  //   readMoreLessStyle: null,
  //   photo: Photo1
  // };
  constructor() {
    super();

    this.state = {
      readMoreLessText: '...Read more',
      readMoreLessStyle: null,
      photo: Photo1
    };
  }

  readMoreLessToggle = () => {
    this.setState({
      readMoreLessText: null,
      readMoreLessStyle: {
        height: 'auto',
        cursor: 'default'
      }
    });
  }

  changePhoto(photo: string) {
    this.setState({photo});
  }

  render() {
    const { readMoreLessStyle, readMoreLessText, photo } = this.state;

    return (
      <div className="FullApp">

        <div id="app-icon">
          <svg x="0px" y="0px" width="512px" height="512px" viewBox="0 0 512 512">
            <linearGradient id="SVGID_1_" gradientUnits="userSpaceOnUse" x1="3.0005" y1="509" x2="508.9999" y2="3.0005">
              <stop offset="0" style={{stopColor:"#48E0CA"}}/>
              <stop offset="0.25" style={{stopColor:"#026BDF"}}/>
              <stop offset="0.5" style={{stopColor:"#605BB1"}}/>
              <stop offset="0.75" style={{stopColor:"#EF4C65"}}/>
              <stop offset="1" style={{stopColor:"#E01583"}}/>
            </linearGradient>
            <path fill="none" stroke="url(#SVGID_1_)" strokeWidth="2.5" strokeMiterlimit="10" d="M3,3l506,506 M3,509L509,3 M35.527,3v506
             M477.394,3v506 M3,34.752h506 M3,475.477h506 M509,117.48C509,54.256,457.749,3,394.522,3H117.481C54.256,3,3,54.256,3,117.48
            V394.52C3,457.746,54.256,509,117.481,509h277.042C457.749,509,509,457.746,509,394.52V117.48z M349.182,256
            c0-51.463-41.716-93.186-93.18-93.186c-51.464,0-93.185,41.723-93.185,93.186c0,51.464,41.721,93.184,93.185,93.184
            C307.466,349.184,349.182,307.464,349.182,256z M388.109,256c0-72.96-59.149-132.107-132.107-132.107
            c-72.96,0-132.106,59.146-132.106,132.107c0,72.961,59.146,132.104,132.106,132.104C328.96,388.104,388.109,328.961,388.109,256z
             M455.568,256c0-110.22-89.348-199.57-199.566-199.57C145.783,56.43,56.431,145.78,56.431,256
            c0,110.223,89.352,199.565,199.571,199.565C366.221,455.565,455.568,366.223,455.568,256z M162.817,3v506 M256.002,3v506 M349.182,3
            v506 M3,162.814h506 M3,256h506 M3,349.184h506"/>
          </svg>
        </div>

        <Switch defaultValue={true} />

        <div className="col-md-8" id="app-content-container">
          <div id="app-header">
            App Header
          </div>

          <div id="app-content" style={readMoreLessStyle}>
            {contentExample.split("\n").map((text, i) => {
                return <div key={i}>{text}&nbsp;</div>;
            })}
          </div>

          <div id="read-more-less" style={readMoreLessStyle} onClick={this.readMoreLessToggle}>{readMoreLessText}</div>

          <div id="photo-container">
            <div id="photo-sized-container">
              <div id="photo-lg"><img src={photo} alt="Photo1" /></div>
              <div id="photo-gallery">
                {/*<div className="photo-border" />*/}
                <div className="photos"      id="first" ><img onClick={this.changePhoto.bind(this,Photo1)} src={Photo1} alt="Photo1" /></div>
                <div className="photo-border" />
                <div className="photos"      id="second"><img onClick={this.changePhoto.bind(this,Photo2)} src={Photo2} alt="Photo2" /></div>
                <div className="photo-border" />
                <div className="photos"      id="third" ><img onClick={this.changePhoto.bind(this,Photo3)} src={Photo3} alt="Photo3" /></div>
                <div className="photo-border" />
                <div className="photos"      id="fourth"><img onClick={this.changePhoto.bind(this,Photo4)} src={Photo4} alt="Photo4" /></div>
                <div className="photo-border" />
                <div className="photos last" id="fifth" ><img onClick={this.changePhoto.bind(this,Photo5)} src={Photo5} alt="Photo5" /></div>
                {/*<div className="photo-border" />*/}
              </div>
            </div>
          </div>
        </div>

        <div className="col-md-4" id="app-information-container">

          <div className="right-side-border" />
          <div className="right-side-header" id="information-content">Information</div>
          <div className="information-content">
            <div className="information-text" id="information-catagory">
              Catagory: Productivity
            </div>
            <div className="information-text" id="information-updated">
              Updated: April 28, 2017
            </div>
            <div className="information-text" id="information-version">
              Version: 2.8.3
            </div>
            <div className="information-text" id="information-language">
              Languages: English, Russian, German, Italian, Japanese
            </div>
            <div className="information-text" id="information-owner">
              Owner: Booker Dewitt
            </div>
          </div>

          <div className="right-side-border top-margin" />
          <div className="right-side-header" id="author">More from the Author</div>
          <div className="information-content">
            {miniApp}
            {miniApp}
            {miniApp}
          </div>

        </div>

      </div>
    );
  }
}

export default FullApp;
