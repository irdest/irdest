import Component from '@glimmer/component';
import { modifier } from 'ember-modifier';
import QRCode from 'qrcode';

function parseAddrs(addrs) {
  return Object.entries(addrs).map((addr) => {
    return { key: addr[0], val: addr[1] };
  });
}

const qr = modifier((canvas, [content]) => {
  QRCode.toCanvas(canvas, content);
});

class Addr extends Component {
  get prettyName() {
    return this.args.addr.key;
  }

  <template>
    <tr>
      <td>{{ this.prettyName }}</td>
      <td><canvas {{ qr this.prettyName }}/></td>
    </tr>
  </template>
}


<template>
  {{# if (parseAddrs @addrs) }}
  <table class="addrs-table">
    {{# each (parseAddrs @addrs) as | addr | }}
    <Addr @addr={{ addr }}/>
    {{/ each }}
  </table>
  {{ else }}
    <p>This router has not encountered any network peers yet</p>
  {{/ if }}
</template>
