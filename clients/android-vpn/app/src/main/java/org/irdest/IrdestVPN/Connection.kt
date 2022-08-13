package org.irdest.IrdestVPN

import android.util.Log
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
import java.nio.channels.FileChannel

class Connection(
    val input: FileChannel,
    val output: FileChannel)
    : ViewModel() {
    private val TAG = Connection::class.java.simpleName

    private val ioDispatcher : CoroutineDispatcher = Dispatchers.IO
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