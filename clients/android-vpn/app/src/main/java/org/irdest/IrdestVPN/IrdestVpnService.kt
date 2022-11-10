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
import java.util.concurrent.atomic.AtomicReference

class IrdestVpnService : VpnService() {
    private val TAG = IrdestVpnService::class.java.simpleName

    private val ratmandThread = AtomicReference<Thread>()

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

    private fun connect() {
        // Become a foreground service.
        updateForegroundNotification(R.string.connecting)

        // Run `ratmand` router
        runRatmandRouter()
    }

    private fun disconnect() {
        setRouterThread(null)
        stopForeground(true)
    }

    private fun runRatmandRouter() {
        Thread(RatmandRouter(), "RatmandThread[]")
            .run {
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
        val NOTIFICATION_CHANNEL_ID = "IrdestVpn"

        // From Android 8.0 notification channel must be created.
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            (getSystemService(NOTIFICATION_SERVICE) as NotificationManager)
                .createNotificationChannel(
                    NotificationChannel(
                        NOTIFICATION_CHANNEL_ID,
                        NOTIFICATION_CHANNEL_ID,
                        NotificationManager.IMPORTANCE_DEFAULT))
        }

        startForeground(2, NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle(NOTIFICATION_CHANNEL_ID)
            .setContentText(getString(msg))
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentIntent(PendingIntent.getActivity(
                this,
                0,
                Intent(this,MainActivity::class.java),
                FLAG_UPDATE_CURRENT
            ))
            .build())
    }
}