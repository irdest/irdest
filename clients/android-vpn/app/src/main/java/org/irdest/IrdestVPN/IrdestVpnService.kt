package org.irdest.IrdestVPN

import android.app.Service
import android.content.Intent
import android.net.VpnService
import org.irdest.IrdestVPN.utils.NOTIFICATION_ID
import org.irdest.IrdestVPN.utils.getNotification
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

class IrdestVpnService : VpnService() {
    private val TAG = IrdestVpnService::class.java.simpleName

    private val connectionID = AtomicInteger(1)
    private val ratmandThread = AtomicReference<Thread>()

    companion object {
        const val ACTION_CONNECT = "IRDEST_VPN_CONNECT"
        const val ACTION_DISCONNECT = "IRDEST_VPN_DISCONNECT"
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        return if (intent?.action == ACTION_DISCONNECT) {
            disconnect()
            Service.START_NOT_STICKY
        } else {
            connect()
            Service.START_STICKY
        }
    }

    private fun disconnect() {
        setRouterThread(null)
        stopForeground(true)
    }

    private fun connect() {
        // Become a foreground service.
        startForeground(NOTIFICATION_ID, getNotification(R.string.connecting, this))

        runRatmand()
    }

    private fun runRatmand() {
        Thread(RatmandRunnable(), "RatmandThread").run {
            setRouterThread(this)
            isDaemon = true
            start()
        }
    }

    private fun setRouterThread(thr: Thread?) {
        // Replace any existing ratmand thread with the new one.
        ratmandThread.getAndSet(thr)?.interrupt()
    }
}