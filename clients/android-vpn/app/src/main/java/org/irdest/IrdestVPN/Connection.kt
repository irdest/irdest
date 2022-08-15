package org.irdest.IrdestVPN

import android.util.Log
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import java.nio.ByteBuffer
import java.nio.channels.FileChannel

class Connection(
    val input: FileChannel,
    val output: FileChannel)
{
    private val TAG = Connection::class.java.simpleName

    private val MAX_PACKET_SIZE = 1024

    private var alive = true

    fun connect() {
        object : Thread() {
            override fun run() {
                Log.d(TAG, "runForever(Irdest-proxy): Current Thread = "
                        + Thread.currentThread())
                while(alive) {
                    val buffer = ByteBuffer.allocate(MAX_PACKET_SIZE)

                    if (input.read(buffer) <= 0) {
                        Thread.sleep(1000)
                    }

                    // Received packet from local.
                    Log.d(TAG, "vpnRunLoop: Received packet from local packet: "
                            + "Bytes = " + buffer.array())

                    buffer.clear()
                }
                Log.d(TAG, "runForever: Main loop is stopped.")
            }
        }.start()
    }

    fun disconnect() {
        alive = false
        input.close()
        output.close()
        Log.d(TAG, "disconnect: FileChannels are closed")
    }
}