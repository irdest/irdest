import Route from '@ember/routing/route';
import { setRouteComponent } from 'experimental-set-route-component';
import MarkdownToHtml from 'ember-cli-showdown/components/markdown-to-html';
import dedent from 'ts-dedent';

class DownloadRoute extends Route {}

const content = dedent`
  Before downloading, please beware that this is **alpha stage
  software** and as such it will have bugs and produce crashes.  Please
  do not expect to be able to rely on this software in your day-to-day
  setup.

  That being said: we want to improve Irdest for everyone so if you
  experience a crash, please report the issue to our [issue
  tracker][issues] or our [community mailing ist][mail]!

  [issues]: https://git.irde.st/we/irdest/-/issues
  [mail]: https://lists.irde.st/archives/list/community@lists.irde.st/

  There are several ways to install Irdest.  Check the instructions for
  your platform below.

  Currently only *Linux* systems running *systemd* are actively being
  tested on!


  ### Distribution packages

  You can find pre-made packages for several Linux and BSD
  distributions.  Consult the following table for details.

  [![Packaging status](https://repology.org/badge/vertical-allrepos/ratman.svg)](https://repology.org/project/ratman/versions)


  ### Portable/ stand-alone binaries

  If you're using a distribution which currently doesn't have a package
  available you can install Irdest/ Ratman via the stand-alone Irdest
  bundle.  The bundle includes all major Irdest applications, an
  installer, and a copy of the user manual.

  - Linux [x86_64 bundle](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.4.0/raw/ratman-bundle-x86_64.tar.gz?job=bundle-ratman)!
  - Linux [aarch64 bundle](https://git.irde.st/we/irdest/-/jobs/artifacts/ratman-0.4.0/raw/ratman-bundle-aarch64.tar.gz?job=bundle-ratman-aarch64)
`;

const Component = <template>
  <main class="block block-content">
    <div class="wrap">
      <div class="title-image"><img src="/img/ratman.png" width="227" height="350" /></div>
      <h1 class="title-header">Irdest Download</h1>

      <MarkdownToHtml @markdown={{content}} />
    </div>
  </main>
</template>;

export default setRouteComponent(Component, DownloadRoute);
