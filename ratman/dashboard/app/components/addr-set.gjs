function parseAddrs(addrs) {
  return Object.entries(addrs).map(([key, val]) => ({ key, val }));
}

<template>
  <table>
    {{# each (parseAddrs @addrs) as | addr | }}
    <tr> {{ addr.key }} |awawa|{{ addr.val }}</tr>
    {{/ each }}
  </table>
</template>
