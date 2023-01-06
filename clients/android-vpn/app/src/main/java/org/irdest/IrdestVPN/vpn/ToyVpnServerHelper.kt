package org.irdest.irdestVPN.vpn

import java.nio.ByteBuffer
import java.nio.channels.DatagramChannel
import java.util.concurrent.TimeoutException

class ToyVpnServerHelper {

    fun handshake(tunnel: DatagramChannel, sharedSecret: String): TunConfigData {
        val packet = getHandshakePacket(sharedSecret)

        // Send packet 3 times in case of packet loss.
        for (i in 1..3) {
            tunnel.write(packet)
        }

        // Wait for the response.
        for (i in 1..MAX_HANDSHAKE_ATTEMPTS) {
            packet.clear()
            Thread.sleep(INTERVAL_MS)

            if (tunnel.read(packet) > 0 && isControlPacket(packet.get(0))) {
                return parseParams(String(packet.array()))
            }
        }
        throw TimeoutException("Handshake time out.")
    }

    private fun getHandshakePacket(sharedSecret: String): ByteBuffer {
        val packet = ByteBuffer.allocate(HANDSHAKE_PACKET_SIZE)
        packet
            .put((PREFIX_CONTROL_PACKET.toString() + sharedSecret).toByteArray())
            .flip()

        return packet
    }

    @Throws
    private fun parseParams(params: String): TunConfigData {
        val trimmedParams = params.trim().drop(1)  // Drop control prefix.

        var address: Pair<String, Int>? = null
        var mtu: Int? = null
        var route: Pair<String, Int>? = null
        var dns: String? = null

        for (param in trimmedParams.trim().split(" ")) {
            val fields = param.split(",")

            when (fields[0].first().toString()) {
                "a" -> address = Pair(fields[1], fields[2].toInt())
                "m" -> mtu = fields[1].toInt()
                "r" -> route = Pair(fields[1], fields[2].toInt())
                "d" -> dns = fields[1]
            }
        }

        return TunConfigData(address!!, route!!, mtu!!, dns!!)
    }

    companion object {
        private const val HANDSHAKE_PACKET_SIZE = 1024
        private const val MAX_HANDSHAKE_ATTEMPTS = 50
        private const val PREFIX_CONTROL_PACKET = 0
        private const val INTERVAL_MS: Long = 100

        fun isControlPacket(firstByte: Byte): Boolean {
            return firstByte.toInt() == PREFIX_CONTROL_PACKET
        }
    }
}