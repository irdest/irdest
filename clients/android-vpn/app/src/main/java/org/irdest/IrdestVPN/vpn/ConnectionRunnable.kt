package org.irdest.irdestVPN.vpn

import android.os.ParcelFileDescriptor
import android.util.Log
import org.irdest.irdestVPN.models.ConnectionState.ERROR
import org.irdest.irdestVPN.utils.TAG
import java.net.InetSocketAddress
import java.nio.channels.DatagramChannel

class ConnectionRunnable(
    private val vpnService: IrdestVpnService,
    private val connectionId: Int,
    private val serverAddress: String,
    private val serverPort: Int,
    private val sharedSecret: String,
) : Runnable {

    interface OnEstablishListener {
        fun onEstablish(tunInterface: ParcelFileDescriptor)
    }

    private lateinit var onEstablishListener: OnEstablishListener

    fun setConnectionEstablishListener(onEstablish: (ParcelFileDescriptor) -> Unit) {
        this.onEstablishListener = object : OnEstablishListener {
            override fun onEstablish(tunInterface: ParcelFileDescriptor) {
                onEstablish(tunInterface)
            }
        }
    }

    override fun run() {
        try {
            Log.i(TAG, "run: Starting new connection [$connectionId].")
            runConnection()
        } catch (e: Exception) {
            when (e) {
                is InterruptedException -> {
                    // User clicked disconnect.
                    Log.i(TAG, "run: Disconnect [$connectionId].")
                    vpnService.onDestroy()
                }
                else -> {
                    vpnService.updateUI(ERROR, "$e${e.message.toString()}")
                }
            }
        }
    }

    private fun runConnection() {
        var tunInterface: ParcelFileDescriptor? = null
        var tunnel: DatagramChannel? = null

        try {
            tunnel = getTunnel()
            Log.i(TAG, "runConnection: Connect to server [$serverAddress : $serverPort]")

            // Authenticate and get configuration values for the Tun interface.
            val tunConfig = handshake(tunnel)

            // Establish the Tun interface.
            tunInterface = getTun(tunConfig)

            synchronized(vpnService) {
                onEstablishListener.onEstablish(tunInterface)
            }

            VpnProcessor().run(tunnel, tunInterface)
        } finally {
            tunInterface?.close()
            tunnel?.disconnect()
        }
    }

    @Throws
    private fun getTunnel(): DatagramChannel {
        // Create a  DatagramChannel as the VPN tunnel.
        val tunnel = DatagramChannel.open()

        if (!vpnService.protect(tunnel.socket())) {
            throw IllegalStateException("Failed to protect the tunnel")
        }

        tunnel.configureBlocking(false) // non-blocking mode.

        return tunnel.connect((InetSocketAddress(serverAddress, serverPort)))
    }

    @Throws
    private fun handshake(tunnel: DatagramChannel): TunConfigData {
        return ToyVpnServerHelper().handshake(tunnel, sharedSecret)
    }

    @Throws
    private fun getTun(tunConfig: TunConfigData): ParcelFileDescriptor {
        val builder = vpnService.Builder()
            .addAddress(tunConfig.Address.first, tunConfig.Address.second)
            .addRoute(tunConfig.Route.first, tunConfig.Route.second)
            .setMtu(tunConfig.MTU)

        when (val tun = builder.establish()) {
            null -> throw IllegalStateException("Failed to establish a Tun with [$tunConfig].")
            else -> return tun
        }
    }

}