import Route from '@ember/routing/route';
import { setRouteComponent } from 'experimental-set-route-component';
import MarkdownToHtml from 'ember-cli-showdown/components/markdown-to-html';
import dedent from 'ts-dedent';

class IndexRoute extends Route {}

const introContent = [{
  title: 'Build resilient networks',
  content: dedent`
    Centralised communication infrastructure is vulnerable to exploitation
    or attacks by natural disasters, oppressive governments, or gate-keepers
    of digital connections (i.e. internet service providers).  _Irdest side-steps
    existing infrastructure_ to allow network participants to take ownership
    of the infrastructure together.  This also makes it much harder to censor or control.
  `,
  image: 'img/cube.svg',
}, {
  title: 'Make use of an extensible architecture',
  content: dedent`
    An Irdest network can be composed of both devices and other
    networks, creating a network of networks; _a new internet_.  With
    the Irdest client SDK third-party applications can interact with
    each other on an Irdest network, or on an alternatively managed
    network bridged via Irdest.
  `,
  image: 'img/extension.svg',
}, {
  title: 'Free and open-source software',
  content: dedent`
  Irdest is not owned by a single company or legal entity.  All code
  is licensed under a free software license.  This means it is free
  for anyone to use, study, and adapt, forever.
  `,
  image: 'img/network.svg',
}];

const introduction = dedent`
  *Irdest is a networking research project** that explores
  different technologies and ideas on how to build more sustainable,
  user-controlled communication networks.

  Whether you are connected to the internet via your home ISP
  (internet service provider) or via a mobile phone network,
  powerful and complex organisations sit between you and your
  ability to communicate with other people.

  As part of an *Irdest network* your home computer, laptop,
  router, phone, etc connect to each other directly, creating a
  *dynamic mesh network*.  This means that the communication
  infrastructure that we collectively rely on to organise ourselves
  needs to in turn become collectively organised and managed.  This
  approach is very different from the "internet service" you usually
  currently buy from a company.

  **A lot of decentralised networking technology already exists!** A
  primary motivation for the Irdest project is to take decades of
  research in this field and make it more accessible to end-users
  and curious software developers alike.

  With the Irdest SDK you can write applications that are *native to
  a decentralised mesh network* and don't require a central server,
  or access to the internet to operate!
`;

const meshing = dedent`
  At the heart of an Irdest network sits Ratman, a *router
  application* that runs on phones, computers, laptops, and other
  devices.  Different Ratman instances can be connected over a wide
  range of connection types.

  Communicating between Ratman instances works seemlessly, the same
  way as devices on a WiFi network can, with the added ability to
  link these networks over long distances or across the entire
  world.

  Connections between Ratman instances can be created via local
  networks, long-range LoRa modems, peer-to-peer Wireless
  connections, or over the internet as a VPN-like network.

  Applications using an Irdest network can discover and connect with
  each other by first connecting to a local Ratman instance.  _The
  range of possible applications using this technology is
  limitless._

  For a more detailed explanation you should check out the
  ["Concepts & Ideas"](https://docs.irde.st/user/concepts.html)
  section in the user manual.
`;

const About = <template>
  <section class="block block-content">
    <div class="wrap">
      <h1>What is Irdest?</h1>
      <MarkdownToHtml @markdown={{introduction}} />

      <h1>How does Irdest work?</h1>
      <MarkdownToHtml @markdown={{meshing}} />
    </div>
  </section>
</template>

const Intro = <template>
  <section class="block block-intro">
    <div class="wrap">
      {{#each introContent as |content|}}
        <div class="intro-item">
          <img src="{{content.image }}" />
          <div class="intro-col">
            <h2>{{content.title}}</h2>
            <MarkdownToHtml @markdown={{content.content}} />
          </div>
        </div>
      {{/each}}
    </div>
  </section>
</template>

const whatNext = dedent`
  Go to the [Download](/download) section of the website to learn
  more about available application bundles, packages, and
  installers.

  We also recommend you read the [user
  manual](https://docs.irde.st/user/) to familiarise yourself with
  some basic technical concepts currently required to operate an
  Irdest network!

  # Questions?

  Check out the [Learn](/learn#manuals) page where we collect
  community resources, manuals, and guides.

  Is your use-case not covered by any manual or guide?  Check out
  the [Community](/community) page to learn how to get in touch
  with us.
`;

const Contribute = <template>
  <section class="block block-contribute">
    <div class="wrap">
      <h1>What next?</h1>
      <MarkdownToHtml @markdown={{whatNext}} />
    </div>
  </section>
</template>

const Funding = <template>
  <section class="block block-content">
    <div class="wrap">
      <h1>Funding \& Partnerships</h1>
      <p>
        The Irdest project does not work alone, and we are always looking for
        collaboration opportunities.  Please don't hesitate to contact us!
      </p>

      <div class="sponsors">
        <a href="https://nlnet.nl/project/Irdest/"><img class="sponsor" src="img/nlnet.svg" /></a>
        <a href="https://nlnet.nl/NGI0/"><img class="sponsor" src="img/NGIZero-green.hex.svg" /></a>
        <a href="https://summerofcode.withgoogle.com/projects/#4792427082153984"><img class="sponsor" src="img/GSoC.svg" /></a>
        <a href="https://freifunk.net"><img class="sponsor" src="img/freifunk.svg" /></a>
      </div>
    </div>
  </section>
</template>

export default setRouteComponent(<template>
  <Intro />
  <About />
  <Contribute />
  <Funding />
</template>, IndexRoute);
