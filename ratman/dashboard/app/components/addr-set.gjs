import Component from '@glimmer/component';
import { tracked } from '@glimmer/tracking';
import { modifier } from 'ember-modifier';
import QRCode from 'qrcode';

function parseAddrs(addrs) {
  return Object.entries(addrs).map((addr) => {
    return { key: addr[0], val: addr[1] };
  });
}

const qr = modifier((canvas, [content]) => {
  QRCode.toCanvas(canvas, content);
  // return a destructor for cleanup optionally
});

class Addr extends Component {
  @tracked qrDataUrl;

  constructor(...args) {
    super(...args);
    this.buildQRDataUrl();
  }

  get prettyName() {
    return this.args.addr.key;
  }

  async buildQRDataUrl() {
    this.qrDataUrl = await QRCode.toDataURL(this.prettyName);
  }

  <template>
    <tr>
      <td>{{ this.prettyName }}</td>
      <td><canvas {{ qr this.prettyName }}/></td>
      <td><img src={{ this.qrDataUrl }} /></td>
    </tr>
  </template>
}


<template>
  <table class="addrs-table">
    {{# each (parseAddrs @addrs) as | addr | }}
      <Addr @addr={{ addr }}/>
    {{/ each }}
  </table>
</template>
