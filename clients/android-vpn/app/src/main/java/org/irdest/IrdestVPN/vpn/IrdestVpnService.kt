package org.irdest.irdestVPN.vpn

import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.Message
import android.os.Messenger
import android.os.ParcelFileDescriptor
import android.util.Log
import org.irdest.irdestVPN.models.ConnectionState
import org.irdest.irdestVPN.models.ConnectionState.*
import org.irdest.irdestVPN.models.VpnState
import org.irdest.irdestVPN.models.isValid
import org.irdest.irdestVPN.R
import org.irdest.irdestVPN.utils.*
import java.util.*
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

class IrdestVpnService : VpnService() {

    private val connectionId = AtomicInteger(1)
    private val atomicThreadRef = AtomicReference<Thread>()
    private val atomicThreadAndTunRef = AtomicReference<Pair<Thread, ParcelFileDescriptor>>()

    private var messenger: Messenger? = null

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return when (val action = intent?.action) {
            ACTION_CONNECT -> {
                messenger = intent.getParcelableExtra(MESSENGER_EXTRA_NAME)
                connect()
                START_STICKY
            }
            ACTION_DISCONNECT -> {
                disconnect()
                START_NOT_STICKY
            }
            else -> {
                receivedUnknownAction(action)
                START_FLAG_RETRY
            }
        }
    }

    override fun onDestroy() {
        disconnect()
    }

    private fun connect() {
        // Become a foreground service.
        startForeground()

        val connection = getConnection()
        if (connection != null) {
            val connectionThread = Thread(connection, "IrdestVPNConnection")
            setAtomicConnectionThread(connectionThread)

            connection.setConnectionEstablishListener { tun ->
                updateUI(CONNECTED)

                atomicThreadRef.compareAndSet(connectionThread, null)
                setAtomicConnectionPair(Pair(connectionThread, tun))
            }

            connectionThread.start()
        }
    }

    private fun startForeground() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            createNotificationChannel(this)
        }

        startForeground(NOTIFICATION_ID, getNotification(R.string.connecting, this))
    }

    private fun getConnection(): ConnectionRunnable? {
        val serverInfo = ServerInfoRepository(this).getServerInfo()

        try {
            serverInfo.isValid()
        } catch (e: InputMismatchException) {
            updateUI(ERROR, e.message.toString())
            return null
        }

        return ConnectionRunnable(
            this,
            connectionId.getAndIncrement(),
            serverInfo.serverAddress,
            serverInfo.serverPort.toInt(),
            serverInfo.sharedSecret,
        )
    }

    private fun setAtomicConnectionThread(thread: Thread?) {
        atomicThreadRef.getAndSet(thread)?.interrupt()
    }

    private fun setAtomicConnectionPair(newPair: Pair<Thread, ParcelFileDescriptor>?) {
        atomicThreadAndTunRef.getAndSet(newPair)?.let {
            it.first.interrupt()
            it.second.close()
        }
    }

    private fun disconnect() {
        setAtomicConnectionThread(null)
        setAtomicConnectionPair(null)
        sendMsgToViewModel(DISCONNECTED)
        messenger = null
        stopForeground(true)
    }

    fun updateUI(state: ConnectionState, msg: String = "") {
        updateNotification(getBaseMessageOfState(state), this@IrdestVpnService)
        sendMsgToViewModel(state, msg)
        vpnState = VpnState(state, msg)
    }

    private fun sendMsgToViewModel(state: ConnectionState, strMsg: String = "") {
        with(Message.obtain()) {
            what = state.code
            obj = strMsg

            messenger?.send(this)
        }
    }

    private fun receivedUnknownAction(action: String?) {
        Log.e(TAG, "receivedUnknownAction: [$action]")
    }

    companion object {
        const val ACTION_CONNECT = "IRDEST_VPN_CONNECT"
        const val ACTION_DISCONNECT = "IRDEST_VPN_DISCONNECT"
        const val MESSENGER_EXTRA_NAME = "IRDEST_VPN_MESSENGER_EXTRA"

        var vpnState: VpnState? = null
    }
}