import Route from '@ember/routing/route';
import { setRouteComponent } from 'experimental-set-route-component';
import MarkdownToHtml from 'ember-cli-showdown/components/markdown-to-html';
import dedent from 'ts-dedent';

class CommunityRoute extends Route {}

const content = dedent`
  Do you have questions about irdest?  Trouble setting up your client?
  Or do you want to help out with development?

  We have an active community on Matrix!  [Join the Matrix
  Space](https://matrix.to/#/#irdest:irde.st).

  Alternatively you can find the most important channels linked directly
  below.  Currently none of the rooms are bridged to IRC.  We are
  working on this!

  | Room address               | Room description                                                                              |
  |----------------------------|-----------------------------------------------------------------------------------------------|
  | [#chat:irde.st]()          | Generaly community chat                                                                       |
  | [#dev-chat:irde.st]()      | Developer chat.  While we talk more about implementation specifics, anyone is welcome to join |
  | [#dev-android:irde.st]()   | Android specific development chat                                                             |
  | [#dev-packaging:irde.st]() | Packaging and distribution specific development chat                                          |

  For long-form discussions, RFCs, general design proposals, or to
  submit patches we also host a [community mailing list][mail].

  The source repository is hosted on [git.irde.st](https://git.irde.st/we/irdest)

  [matrix]: https://matrix.to/#/#irdest:fairydust.space?via=ontheblueplanet.com&via=matrix.org&via=fairydust.space
  [mail]: https://lists.irde.st/archives/list/community@lists.irde.st/
`;

const Component = <template>
  <main class="block block-content">
    <div class="wrap">
      <div class="title-image"><img src="/img/tobi_delivers_xtra.png" height="350" /></div>
      <h1 class="title-header">Irdest Community</h1>

      <MarkdownToHtml @markdown={{content}} />
    </div>
  </main>
</template>;

export default setRouteComponent(Component, CommunityRoute);
