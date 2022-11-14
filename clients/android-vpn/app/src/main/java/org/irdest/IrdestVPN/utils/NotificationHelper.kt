package org.irdest.IrdestVPN.utils

import android.app.Notification
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import androidx.core.app.NotificationCompat
import org.irdest.IrdestVPN.MainActivity
import org.irdest.IrdestVPN.R

class NotificationHelper(
    val context: Context
) {
    val NOTIFICATION_CHANNEL_ID = "IrdestVpn"

    fun getNotification(msg: Int) : Notification {
        return getNotificationBuilder()
            .setContentText(context.getString(msg))
            .build()
    }

    private fun getNotificationBuilder() : NotificationCompat.Builder {
        return NotificationCompat
            .Builder(context, NOTIFICATION_CHANNEL_ID)
            .setContentTitle(NOTIFICATION_CHANNEL_ID)
            .setSmallIcon(R.drawable.ic_launcher_foreground)
            .setContentIntent(getPendingIntent())
    }

    private fun getPendingIntent() : PendingIntent {
        return PendingIntent
            .getActivity(
                context,
                0,
                Intent(context, MainActivity::class.java),
                PendingIntent.FLAG_UPDATE_CURRENT
            )
    }
}