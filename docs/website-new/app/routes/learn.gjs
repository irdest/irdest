import Route from '@ember/routing/route';
import { setRouteComponent } from 'experimental-set-route-component';
import MarkdownToHtml from 'ember-cli-showdown/components/markdown-to-html';
import dedent from 'ts-dedent';

class LearnRoute extends Route {}

const intro = dedent`
  This page outlines available Irdest documentation for both users
  and developers.  If you are new to Irdest in general we recommend
  you read the user manual first, even if you also want to develop
  applications for Irdest.

  We also want to list other community resources that are available
  either here, or on the community page.  If you wrote a guide,
  tutorial, or similar, please get in touch!
`;

const users = dedent`
  This manual is aimed at end-users of irdest.  It guides you through
  setting up various clients on your computer, and provides additional
  debugging help and FAQs.

  If you come across an issue not covered by the manual, feel free to join
  one of our [community](/community) channels to ask for help!

  * [User Manual](https://docs.irde.st/user/)
`;

const devs = dedent`
  If you want to contribute to the irdest ecosystem, the following
  resources contain information on where to start.  The Rust docs
  outline the main library APIs and how to use various components
  together.

  The developer manual gives a broad overview of concepts,
  components, and protocols in use by Irdest internals.

  * [Developer Manual](https://docs.irde.st/developer/)
  * [Ratman API docs](https://docs.rs/ratman-client/)
  * [Bibliography](https://docs.irde.st/developer/technical/bib.html)
`

export default setRouteComponent(<template>
  <main class="block block-content">
    <div class="wrap">
      <h1></h1>
      <div class="title-image"><img src="/img/tobi_reads_xtra.png" width="227" height="350" /></div>
      <h1 class="title-header"> Learn about Irdest </h1>

      <p><MarkdownToHtml @markdown={{intro}} /></p>

      <div class="twocol">
        <div class="col-padding">
          <h2>For users</h2>
          <MarkdownToHtml @markdown={{users}} />
        </div>
        <div class="col-padding">
          <h2>For developers</h2>
          <MarkdownToHtml @markdown={{devs}} />
        </div>
      </div>
    </div>
  </main>
</template>, LearnRoute);
