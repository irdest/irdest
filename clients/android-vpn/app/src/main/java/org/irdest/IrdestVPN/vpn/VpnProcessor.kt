package org.irdest.irdestVPN.vpn

import android.os.ParcelFileDescriptor
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import java.io.FileInputStream
import java.io.FileOutputStream
import java.nio.ByteBuffer
import java.nio.channels.DatagramChannel
import java.nio.channels.FileChannel

private const val MAX_PACKET_SIZE = Short.MAX_VALUE.toInt()

class VpnProcessor {

    private val packet = ByteBuffer.allocate(MAX_PACKET_SIZE)

    fun run(
        tunnel: DatagramChannel,
        tun: ParcelFileDescriptor
    ): Unit = runBlocking {
        // Packets to be sent are queued in this input stream.
        val inputStream = FileInputStream(tun.fileDescriptor).channel
        // Packets received need to be written to this output stream.
        val outputStream = FileOutputStream(tun.fileDescriptor)

        while (true) {
            // Handle the outgoing packet.
            readFromTunWriteToTunnel(inputStream, tunnel)
            // Handle the incoming packet.
            readFromTunnelWriteToTun(tunnel, outputStream)
        }
    }

    private suspend fun readFromTunWriteToTunnel(
        inputStream: FileChannel,
        tunnel: DatagramChannel
    ) = coroutineScope {
        launch {
            inputStream.read(packet).let { bytesRead ->
                if (bytesRead > 0) {
                    writePacketToTunnel(packet, tunnel)
                }
            }
        }
    }

    private suspend fun readFromTunnelWriteToTun(
        tunnel: DatagramChannel,
        outputStream: FileOutputStream,
    ) = coroutineScope {
        launch {
            tunnel.read(packet).let { bytesRead ->
                if (bytesRead > 0 && !isControlPacket(packet.get(0))) {
                    writePacketToTun(packet, outputStream, bytesRead)
                }
            }
        }
    }

    private fun writePacketToTunnel(packet: ByteBuffer, tunnel: DatagramChannel) {
        packet.flip()
        tunnel.write(packet)
        packet.clear()
    }

    private fun writePacketToTun(packet: ByteBuffer, outputStream: FileOutputStream, length: Int) {
        outputStream.write(packet.array(), 0, length)
        packet.clear()
    }

    private fun isControlPacket(firstByre: Byte): Boolean =
        ToyVpnServerHelper.isControlPacket(firstByre)
}