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
import { Checkbox } from 'material-ui';

import styles from '../firstRun.css';

export default class TnC extends Component {
  static propTypes = {
    hasAccepted: PropTypes.bool.isRequired,
    onAccept: PropTypes.func.isRequired
  }

  render () {
    const { hasAccepted, onAccept } = this.props;

    return (
      <div className={ styles.tnc }>
        <h1>SECURITY WARNINGS</h1>
        <ul>
          <li>You are responsible for your own computer security. If your machine is compromised you will lose your ether, access to any contracts and maybe more.</li>
          <li>You are responsible for your own actions. If you mess something up or break any laws while using this software, it is your fault, and your fault only.</li>
        </ul>

        <h1>LEGAL WARNING</h1>
        <h2>SHORT VERSION</h2>
        <h3>Disclaimer of Liability and Warranties</h3>
        <ul>
          <li>The user expressly knows and agrees that the user is using Parity at the user’s sole risk.</li>
          <li>The user represents that the user has an adequate understanding of the risks, usage and intricacies of cryptographic tokens and blockchain-based open source software, eth platform and eth.</li>
          <li>The user acknowledges and agrees that, to the fullest extent permitted by any applicable law, the disclaimers of liability contained herein apply to any and all damages or injury whatsoever caused by or related to risks of, use of, or inability to use, Parity under any cause or action whatsoever of any kind in any jurisdiction, including, without limitation, actions for breach of warranty, breach of contract or tort (including negligence) and that Eth Core Limited shall be not liable for any indirect, incidental, special, exemplary or consequential damages, including for loss of profits, goodwill or data.</li>
          <li>Some jurisdictions do not allow the exclusion of certain warranties or the limitation or exclusion of liability for certain types of damages. Therefore, some of the above limitations in this section may not apply to a user. In particular, nothing in these terms shall affect the statutory rights of any user or exclude injury arising from any wilful misconduct or fraud of Eth Core Limited.</li>
          <li>All rights reserved by Ethcore. Licensed to the public under the GPL v3 <a href='https://www.gnu.org/licenses/gpl-3.0.txt' target='_blank'>https://www.gnu.org/licenses/gpl-3.0.txt</a></li>
        </ul>

        <h2>LONG VERSION</h2>
        <p>The following Terms and Conditions (“Terms”) govern the use of Parity Technologies Limited’s open source software product (“Parity”). Prior to any use of the Parity or any of Parity Technologies Limited’s products (“EthCore’s Products”), the user or anyone on whose behalf the software is used for directly or indirectly (“User”) confirms that they understand and expressly agree to all of the Terms. All capitalized terms in this agreement will be given the same effect and meaning as in the Terms. The group of developers and other personnel that is now, or will be, employed by, or contracted with, or affiliated with, Parity Technologies Limited (“EthCore”) is termed the “EthCore Team”.</p>

        <h3>Acknowledgement of Risks</h3>
        <p>The user acknowledges the following serious risks to any use Parity and expressly agrees not to hold liable EthCore or the EthCore Team should any of these risks occur:</p>

        <h3>Risk of Security Weaknesses in the Parity Core Infrastructure Software</h3>
        <p>Parity rests on open-source software, and although it is professionally developed in line with industry standards (which include external audits of the code base), there is a risk that Ethcore or the Ethcore Team, may have introduce unintentional weaknesses or bugs into the core infrastructural elements of Parity causing the system to lose ETH stored in one or more User accounts or other accounts or lose sums of other valued tokens.</p>

        <h3>Risk of Weaknesses or Exploitable Breakthroughs in the Field of Cryptography</h3>
        <p>Cryptography is an art, not a science. And the state of the art can advance over time Advances in code cracking, or technical advances such as the development of quantum computers, could present risks to cryptocurrencies and Parity, which could result in the theft or loss of ETH. To the extent possible, EthCore intends to update the protocol underlying Parity to account for any advances in cryptography and to incorporate additional security measures, but it cannot predict the future of cryptography or guaranty that any security updates will be made, timely or successful.</p>

        <h3>Risk of Ether Mining Attacks</h3>
        <p>As with other cryptocurrencies, the blockchain used by Parity is susceptible to mining attacks, including but not limited to double-spend attacks, majority mining power attacks, “selfish-mining” attacks, and race condition attacks. Any successful attacks present a risk to the Ethereum ecosystem, expected proper execution and sequencing of ETH transactions, and expected proper execution and sequencing of contract computations. Despite the efforts of the EthCore and the EthCore Team, known or novel mining attacks may be successful.</p>

        <h3>Risk of Rapid Adoption and Insufficiency of Computational Application Processing Power on the Ethereum Network</h3>
        <p>If Parity is rapidly adopted, the demand for transaction processing and distributed application computations could rise dramatically and at a pace that exceeds the rate with which ETH miners can bring online additional mining power. Under such a scenario, the entire Ethereum ecosystem could become destabilized, due to the increased cost of running distributed applications. In turn, this could dampen interest in the Ethereum ecosystem and ETH. Insufficiency of computational resources and an associated rise in the price of ETH could result in businesses being unable to acquire scarce computational resources to run their distributed applications. This would represent revenue losses to businesses or worst case, cause businesses to cease operations because such operations have become uneconomical due to distortions in the crypto-economy.</p>

        <h3>Risk of temporary network incoherence</h3>
        <p>We recommend any groups handling large or important transactions to maintain a voluntary 24 hour waiting period on any ether deposited. In case the integrity of the network is at risk due to issues in the clients, we will endeavour to publish patches in a timely fashion to address the issues. We will endeavour to provide solutions within the voluntary 24 hour waiting period.</p>

        <h3>Use of Parity by you</h3>
        <p>You agree to use Party only for purposes that are permitted by (a) these Terms and (b) any applicable law, regulation or generally accepted practices or guidelines in the relevant jurisdictions (including any laws regarding the export of data or software to and from the United Kingdom or other relevant countries).</p>
        <p>You agree that you will not engage in any activity that interferes with or disrupts Parity’s or EthCore’s Products’ functioning (or the networks which are connected to Parity).</p>
        <p>Unless you have been specifically permitted to do so in a separate agreement with EthCore, you agree that you will not reproduce, duplicate, copy, sell, trade or resell the EthCore’s Products for any purpose unless than in accordance to the terms of the software licence terms available here: <a href='https://www.gnu.org/licenses/gpl-3.0.txt' target='_blank'>https://www.gnu.org/licenses/gpl-3.0.txt</a> (“Software Licence Terms”).</p>
        <p>You agree that you are solely responsible for (and that EthCore has no responsibility to you or to any third party for) any breach of your obligations under these terms and for the consequences (including any loss or damage which EthCore may suffer) of any such breach.</p>

        <h3>Privacy and your personal information</h3>
        <p>You agree to the use of your data (if any is gathered) in accordance with EthCore’s privacy policies: <a href='https://ethcore.io/legal.html' target='_blank'>https://ethcore.io/legal.html</a>. This policy explains how EthCore treats your personal information (if any is gathered), and protects your privacy, when you use EthCore’s Products.</p>

        <h3>Content in Parity</h3>
        <p>You understand that all information and data (such as smart contracts, data files, written text, computer software, music, audio files or other sounds, photographs, videos or other images) which you may have access to as part of, or through your use of, EthCore’s Product are the sole responsibility of the person from which such content originated. All such information is referred to below as the “Content”.</p>
        <p>You should be aware that Content presented to you through Parity or EthCore’s Product may be protected by intellectual property rights which are owned by thisrd parties who may provide that Content to EthCore (or by other persons or companies on their behalf). You may not modify, rent, lease, loan, sell, distribute or create derivative works based on this Content (either in whole or in part) unless you have been specifically told that you may do so by Ethcore or by the owners of that Content, in a separate agreement.</p>
        <p>You understand that by using Parity or EthCore’s Products you may be exposed to Content that you may find offensive, indecent or objectionable and that, in this respect, you use Parity or EthCore’s Products at your own risk.</p>
        <p>You agree that you are solely responsible for (and that EthCore has no responsibility to you or to any third party for) any Content that you create, transmit or display while using Parity or EthCore’s Products and for the consequences of your actions (including any loss or damage which EthCore may suffer) by doing so.</p>

        <h3>Proprietary rights</h3>
        <p>You acknowledge and agree that EthCore own all legal right, title and interest in and to the Parity and EthCore’s Products, including any intellectual property rights which subsist in Parity and EthCore’s Products (whether those rights happen to be registered or not, and wherever in the world those rights may exist).</p>
        <p>Unless you have agreed otherwise in writing with EthCore, nothing in the Terms gives you a right to use any of EthCore’s trade names, trade marks, service marks, logos, domain names, and other distinctive brand features.</p>
        <p>If you have been given an explicit right to use any of these brand features in a separate written agreement with EthCore, then you agree that your use of such features shall be in compliance with that agreement, any applicable provisions of these terms, and EthCore’s brand feature use guidelines as updated from time to time. These guidelines can be viewed online at <a href='https://ethcore.io/press.html' target='_blank'>https://ethcore.io/press.html</a>.</p>
        <p>EthCore acknowledges and agrees that it obtains no right, title or interest from you (or your licensors) under these terms in or to any content that you submit, post, transmit or display on, or through, Parity, including any intellectual property rights which subsist in that content (whether those rights happen to be registered or not, and wherever in the world those rights may exist). Unless you have agreed otherwise in writing with EthCore, you agree that you are responsible for protecting and enforcing those rights and that EthCore has no obligation to do so on your behalf.</p>
        <p>You agree that you shall not remove, obscure, or alter any proprietary rights notices (including copyright and trade mark notices) which may be affixed to or contained within Parity or EthCore’s Products.</p>
        <p>Unless you have been expressly authorized to do so in writing by EthCore, you agree that in using Parity, you will not use any trade mark, service mark, trade name, logo of any company or organization in a way that is likely or intended to cause confusion about the owner or authorized user of such marks, names or logos.</p>

        <h3>License Restrictions from EthCore</h3>
        <p>You may not (and you may not permit anyone else to) copy, modify, create a derivative work of, reverse engineer, decompile or otherwise attempt to extract the source code of the Parity, EthCore’s Products or any part thereof, unless this is expressly permitted by our Software Licence Terms or required by law, or unless you have been specifically told that you may do so by EthCore, in writing.</p>
        <p>Unless EthCore has given you specific written permission to do so, you may not assign (or grant a sub-licence of) your rights to use EthCore’s Products, grant a security interest in or over your rights to use the EthCore’s Products, or otherwise transfer any part of your rights to use the EthCore’s Products.</p>

        <h3>Content licence from you</h3>
        <p>You retain copyright and any other rights you already hold in content which you submit, post or display on or through, Parity.</p>

        <h3>Ending your relationship with EthCore</h3>
        <p>The Terms will continue to apply until terminated by either you or EthCore as set out below.</p>
        <p>EthCore may at any time, terminate its legal agreement with you if:</p>
        <ol>
          <li>you have breached any provision of these Terms (or have acted in manner which clearly shows that you do not intend to, or are unable to comply with the provisions of these terms); or</li>
          <li>EthCore is required to do so by law (for example, where the provision of EthCore’s Product to you is, or becomes, unlawful); or</li>
          <li>the partner with whom EthCore offered products or services to you has terminated its relationship with EthCore or ceased to offer products or services to you; or</li>
          <li>EthCore is transitioning to no longer providing products or services to users in the country in which you are resident or from which you use the service; or</li>
          <li>the provision of products or services to you by EthCore is, in EthCore’s opinion, no longer commercially viable.</li>
          <li>When these Terms come to an end, all of the legal rights, obligations and liabilities that you and EthCore have benefited from, been subject to (or which have accrued over time whilst the Terms have been in force) or which are expressed to continue indefinitely, shall be unaffected by this cessation, and the England and Wales jurisdiction choice shall continue to apply to such rights, obligations and liabilities indefinitely.</li>
        </ol>

        <h3>ACKNOWLEDGEMENT AND ACCEPTANCE OF ALL RISKS, EXCLUSION OF WARRANTIES</h3>
        <p>THE USER EXPRESSLY KNOWS AND AGREES THAT THE USER IS USING PARITY OR ETHCORE’S PRODUCTS AT THE USER’S SOLE RISK. THE USER REPRESENTS THAT THE USER HAS AN ADEQUATE UNDERSTANDING OF THE RISKS, USAGE AND INTRICACIES OF CRYPTOGRAPHIC TOKENS AND BLOCKCHAIN-BASED OPEN SOURCE SOFTWARE, PARITY.</p>
        <p>YOU EXPRESSLY UNDERSTAND AND AGREE THAT YOUR USE OF ETHCORE’S PRODUCTS IS AT YOUR SOLE RISK AND THAT ETHCORE’S PRODUCTS ARE PROVIDED "AS IS" AND “AS AVAILABLE.”</p>
        <p>IN PARTICULAR, ETHCORE, ITS SUBSIDIARIES AND AFFILIATES, AND ITS LICENSORS DO NOT REPRESENT OR WARRANT TO YOU THAT:</p>
        <p>(A) YOUR USE OF PARITY OR ETHCORE’S PRODUCTS WILL MEET YOUR REQUIREMENTS,</p>
        <p>(B) YOUR USE OF PARITY OR ETHCORE’S PRODUCTS WILL BE UNINTERRUPTED, TIMELY, SECURE OR FREE FROM ERROR,</p>
        <p>(C) ANY INFORMATION OBTAINED BY YOU AS A RESULT OF YOUR USE OF PARITY OR ETHCORE’S PRODUCTS WILL BE ACCURATE OR RELIABLE, AND</p>
        <p>(D) THAT DEFECTS IN THE OPERATION OR FUNCTIONALITY OF ANY SOFTWARE PROVIDED TO YOU AS PART OF ETHCORE’S PRODUCTS WILL BE CORRECTED.</p>
        <p>ANY MATERIAL DOWNLOADED OR OTHERWISE OBTAINED THROUGH THE USE OF PARITY OR ETHCORE’S PRODUCTS IS DONE AT YOUR OWN DISCRETION AND RISK AND THAT YOU WILL BE SOLELY RESPONSIBLE FOR ANY DAMAGE TO YOUR COMPUTER SYSTEM OR OTHER DEVICE OR LOSS OF DATA OR ECONOMIC LOSS THAT RESULTS FROM THE DOWNLOAD OF ANY SUCH MATERIAL.</p>
        <p>NO ADVICE OR INFORMATION, WHETHER ORAL OR WRITTEN, OBTAINED BY YOU FROM ETHCORE OR THROUGH OR FROM ETHCORE’S PRODUCTS SHALL CREATE ANY WARRANTY NOT EXPRESSLY STATED IN THE TERMS.</p>
        <p>ETHCORE FURTHER EXPRESSLY DISCLAIMS ALL WARRANTIES AND CONDITIONS OF ANY KIND, WHETHER EXPRESS OR IMPLIED, INCLUDING, BUT NOT LIMITED TO THE IMPLIED WARRANTIES AND CONDITIONS OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT.</p>

        <h3>EXCLUSION AND LIMITATION OF LIABILITY</h3>
        <p>THE USER ACKNOWLEDGES AND AGREES THAT, TO THE FULLEST EXTENT PERMITTED BY ANY APPLICABLE LAW, THE DISCLAIMERS AND EXCLUSION OF LIABILITY CONTAINED HEREIN APPLY TO ANY AND ALL DAMAGES OR INJURY WHATSOEVER CAUSED BY OR RELATED TO RISKS OF, USE OF, OR INABILITY TO USE, PARITY UNDER ANY CAUSE OF ACTION WHATSOEVER OF ANY KIND IN ANY JURISDICTION, INCLUDING, WITHOUT LIMITATION, ACTIONS FOR BREACH OF WARRANTY, BREACH OF CONTRACT OR TORT (INCLUDING NEGLIGENCE) AND THAT NEITHER ETHCORE NOR THE ETHCORE TEAM SHALL BE LIABLE FOR ANY INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY OR CONSEQUENTIAL DAMAGES, INCLUDING FOR LOSS OF PROFITS, GOODWILL OR DATA.</p>
        <p>SOME JURISDICTIONS DO NOT ALLOW THE EXCLUSION OF CERTAIN WARRANTIES OR THE LIMITATION OR EXCLUSION OF LIABILITY FOR CERTAIN TYPES OF DAMAGES. THEREFORE, SOME OF THE ABOVE LIMITATIONS IN THIS SECTION MAY NOT APPLY TO A USER. IN PARTICULAR, NOTHING IN THESE TERMS SHALL AFFECT THE STATUTORY RIGHTS OF ANY USER OR EXCLUDE INJURY ARISING FROM ANY WILLFUL MISCONDUCT OR FRAUD OF ETHCORE.</p>
        <p>SUBJECT TO ANY LIABILITY WHICH MAY NOT BE EXCLUDED, YOU EXPRESSLY UNDERSTAND AND AGREE THAT ETHCORE, ITS SUBSIDIARIES AND AFFILIATES, AND ITS LICENSORS SHALL NOT BE LIABLE TO YOU FOR:</p>
        <p>(A) ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL CONSEQUENTIAL OR EXEMPLARY DAMAGES WHICH MAY BE INCURRED BY YOU, HOWEVER CAUSED AND UNDER ANY THEORY OF LIABILITY. THIS SHALL INCLUDE, BUT NOT BE LIMITED TO, ANY LOSS OF PROFIT (WHETHER INCURRED DIRECTLY OR INDIRECTLY), ANY LOSS OF GOODWILL OR BUSINESS REPUTATION, ANY LOSS OF DATA SUFFERED, COST OF PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES, OR OTHER INTANGIBLE LOSS;</p>
        <p>(B) ANY LOSS OR DAMAGE WHICH MAY BE INCURRED BY YOU, INCLUDING BUT NOT LIMITED TO LOSS OR DAMAGE AS A RESULT OF:</p>
        <p>(I) ANY RELIANCE PLACED BY YOU ON THE COMPLETENESS, ACCURACY OR EXISTENCE OF ANY ADVERTISING, OR AS A RESULT OF ANY RELATIONSHIP OR TRANSACTION BETWEEN YOU AND ANY ADVERTISER OR SPONSOR WHOSE ADVERTISING APPEARS ON ETHCORE’S PRODUCTS;</p>
        <p>(II) ANY CHANGES WHICH ETHCORE MAY MAKE TO ETHCORE’S PRODUCTS, OR FOR ANY PERMANENT OR TEMPORARY CESSATION IN THE PROVISION OF ETHCORE’S PRODUCTS (OR ANY FEATURES WITHIN ETHCORE’S PRODUCTS);</p>
        <p>(III) THE DELETION OF, CORRUPTION OF, OR FAILURE TO STORE, ANY CONTENT AND OTHER COMMUNICATIONS DATA MAINTAINED OR TRANSMITTED BY OR THROUGH YOUR USE OF ETHCORE’S PRODUCTS;</p>
        <p>(IV) YOUR FAILURE TO PROVIDE ETHCORE WITH ACCURATE ACCOUNT INFORMATION (IF THIS IS REQUIRED);</p>
        <p>(V) YOUR FAILURE TO KEEP YOUR PASSWORD OR ACCOUNT DETAILS SECURE AND CONFIDENTIAL;</p>
        <p>THE LIMITATIONS ON ETHCORE’S LIABILITY TO YOU SHALL APPLY WHETHER OR NOT ETHCORE HAS BEEN ADVISED OF OR SHOULD HAVE BEEN AWARE OF THE POSSIBILITY OF ANY SUCH LOSSES ARISING.</p>

        <h3>Copyright and trade mark policies</h3>
        <p>It is EthCore’s policy to respond to notices of alleged copyright infringement that comply with applicable international intellectual property law (including, in the United States, the Digital Millennium Copyright Act) and if EthCore is put on notice and it is under EthCore’s control and terminating the accounts of repeat infringers.</p>

        <h3>Other content</h3>
        <p>Services provided may include hyperlinks to other web sites, smart contracts or content or resources. EthCore may have no control over any web sites or resources which are provided by companies or persons other than EthCore.</p>
        <p>You acknowledge and agree that EthCore is not responsible for the availability of any such external sites or resources, and does not endorse any advertising, products or other materials on or available from such web sites or resources.</p>
        <p>You acknowledge and agree that EthCore is not liable for any loss or damage which may be incurred by you as a result of the availability of those external sites or resources, or as a result of any reliance placed by you on the completeness, accuracy or existence of any advertising, products or other materials on, or available from, such web sites or resources.</p>

        <h3>Changes to the Terms</h3>
        <p>EthCore may make changes to these from time to time. When these changes are made, EthCore will make a new copy of these terms available at https://ethcore.io/legal.html and any new terms will be made available to you from within, or through, the affected EthCore’s Product.</p>
        <p>You understand and agree that if you use Parity or EthCore’s Products after the date on which the Terms have changed, EthCore will treat your use as acceptance of the updated terms.</p>

        <h3>General legal terms</h3>
        <p>Sometimes when you use Parity or EthCore’s Products, you may (as a result of, or in connection with your use of these products) use a service or download a piece of software, or smart contract, or purchase goods, which are provided by another person or company. Your use of these other services, software, smart contract or goods may be subject to separate terms between you and the company or person concerned. If so, these Terms do not affect your legal relationship with these other companies or individuals.</p>
        <p>These Terms constitute the whole legal agreement between you and EthCore and govern your use of Parity and EthCore’s Products (but excluding any products or services which EthCore may provide to you under a separate written agreement), and completely replace any prior agreements between you and EthCore in relation to Parity and EthCore’s Products.</p>
        <p>You agree that EthCore may provide you with notices, including those regarding changes to the Terms, by postings on the affected EthCore’s Product.</p>
        <p>You agree that if EthCore does not exercise or enforce any legal right or remedy which is contained in these Terms (or which EthCore has the benefit of under any applicable law), this will not be taken to be a formal waiver of EthCore’s rights and that those rights or remedies will still be available to EthCore.</p>
        <p>If any court of law, having the jurisdiction to decide on this matter, rules that any provision of these Terms is invalid, then that provision will be removed from the Terms without affecting the rest of the Terms. The remaining provisions of the Terms will continue to be valid and enforceable.</p>
        <p>You acknowledge and agree that each member of the group of companies of which EthCore is the parent shall be third party beneficiaries to these Terms and that such other companies shall be entitled to directly enforce, and rely upon, any provision of the Terms which confers a benefit on (or rights in favor of) them. Other than this, no other person or company shall be third party beneficiaries to these Terms.</p>
        <p>These Terms, and your relationship with EthCore under these Terms, shall be governed by the laws of England and Wales, United Kingdom without regard to its conflict of laws provisions. You and EthCore agree to submit to the exclusive jurisdiction of the courts located within England, United Kingdom to resolve any legal matter arising from these Terms (subject to the Dispute Resolution clause below). Notwithstanding this, you agree that EthCore shall still be allowed to apply for injunctive remedies (or an equivalent type of urgent legal relief) in any jurisdiction.</p>

        <h3>Dispute Resolution</h3>
        <p>All disputes or claims arising out of, relating to, or in connection with the Terms, the breach thereof, or use of Parity shall be finally settled under the Rules of Arbitration of the International Chamber of Commerce by one or more arbitrators appointed in accordance with said Rules. All claims between the parties relating to these Terms that are capable of being resolved by arbitration, whether sounding in contract, tort, or otherwise, shall be submitted to ICC arbitration. Prior to commencing arbitration, the parties have a duty to negotiate in good faith and attempt to resolve their dispute in a manner other than by submission to ICC arbitration. The arbitration panel shall consist of one arbitrator only, unless the ICC Court of Arbitration determines that the dispute is such as to warrant three arbitrators. If the Court determines that one arbitrator is sufficient, then such arbitrator shall be a UK resident. If the Court determines that three arbitrators are necessary, then each party shall have 30 days to nominate an arbitrator of its choice - in the case of the Claimant, measured from receipt of notification of the ICC Court’s decision to have three arbitrators; in the case of Respondent, measured from receipt of notification of Claimant’s nomination. All nominations must be UK residents. If a party fails to nominate an arbitrator, the Court will do so. The Court shall also appoint the chairman. All arbitrators shall be and remain “independent” of the parties involved in the arbitration. The place of arbitration shall be England, United Kingdom. The language of the arbitration shall be English. In deciding the merits of the dispute, the tribunal shall apply the laws of England and Wales and any discovery shall be limited and shall not involve any depositions or any other examinations outside of a formal hearing. The tribunal shall not assume the powers of amiable compositeur or decide the case ex aequo et bono. In the final award, the tribunal shall fix the costs of the arbitration and decide which of the parties shall bear such costs in what proportion. Every award shall be binding on the parties. The parties undertake to carry out the award without delay and waive their right to any form of recourse against the award in so far as such waiver can validly be made.</p>

        <h3>Additional Terms for Enterprise Use</h3>
        <p>If you are a business entity, then the individual accepting on behalf of the entity (for the avoidance of doubt, for business entities, in these Terms, "you" means the entity) represents and warrants that he or she has the authority to act on your behalf, that you represent that you are duly authorized to do business in the country or countries where you operate, and that your employees, officers, representatives, and other agents accessing EthCore’s Products are duly authorized to access Parity and to legally bind you to these Terms.</p>
        <p>Subject to these Terms and subject to the Software Licence Terms, EthCore grants you a non-exclusive, non-transferable licence to install and use Parity solely on machines intended for use by your employees, officers, representatives, and agents in connection with your business entity, and provided that their use of EthCore will be subject to these Terms and EthCore’s Products software licence terms.</p>

        <Checkbox
          className={ styles.accept }
          label={
            <FormattedMessage
              id='firstRun.tnc.accept'
              defaultMessage='I accept these terms and conditions'
            />
          }
          checked={ hasAccepted }
          onCheck={ onAccept }
        />
      </div>
    );
  }
}
