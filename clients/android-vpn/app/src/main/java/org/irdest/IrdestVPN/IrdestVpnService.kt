package org.irdest.IrdestVPN

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.PendingIntent.FLAG_UPDATE_CURRENT
import android.app.Service
import android.content.Intent
import android.net.VpnService
import android.os.Build
import androidx.core.app.NotificationCompat
import org.irdest.IrdestVPN.utils.NotificationHelper
import java.util.concurrent.atomic.AtomicInteger
import java.util.concurrent.atomic.AtomicReference

class IrdestVpnService : VpnService() {
    private val TAG = IrdestVpnService::class.java.simpleName

    private val connectionID = AtomicInteger(1)
    private val ratmandThread = AtomicReference<Thread>()

    private val notificationHelper: NotificationHelper = NotificationHelper(this)

    companion object {
        const val ACTION_CONNECT = "IRDEST_VPN_CONNECT"
        const val ACTION_DISCONNECT = "IRDEST_VPN_DISCONNECT"
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == ACTION_DISCONNECT) {
            disconnect()
            return Service.START_NOT_STICKY
        } else {
            connect()
            return Service.START_STICKY
        }
    }

    private fun disconnect() {
        setRouterThread(null)
        stopForeground(true)
    }

    private fun connect() {
        // Become a foreground service.
        updateForegroundNotification(R.string.connecting)

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

    private fun updateForegroundNotification(msg: Int) {
        val notification = notificationHelper.getNotification(msg)
        startForeground(2, notification)
    }
}