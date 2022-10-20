package org.irdest.IrdestVPN

import android.annotation.SuppressLint
import android.app.*
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.ParcelFileDescriptor
import android.util.Log
import androidx.annotation.RequiresApi
import androidx.core.app.NotificationCompat
import java.io.FileInputStream
import java.io.FileOutputStream
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
        runRatmand()
    }

    private fun disconnect() {
        ratmandThread.get().interrupt()
        stopForeground(true)
    }

    private fun runRatmand() {
        Thread(Ratmand(), "RatmandThread").run {
            isDaemon = true
            stackTrace
            uncaughtExceptionHandler = exceptionHandler
            setRatmandThread(this)
            start()
        }
    }

    private fun setRatmandThread(thr: Thread) {
        ratmandThread.getAndSet(thr)?.interrupt()
    }

    val exceptionHandler = object : Thread.UncaughtExceptionHandler {
        override fun uncaughtException(t: Thread, e: Throwable) {
            Log.e(TAG, "uncaughtExceptionHandler: ", e)
        }
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
            .setSmallIcon(androidx.loader.R.drawable.notification_bg)
            .setContentIntent(PendingIntent.getActivity(
                this,
                0,
                Intent(this,MainActivity::class.java),
                PendingIntent.FLAG_UPDATE_CURRENT))
            .build())
    }
}