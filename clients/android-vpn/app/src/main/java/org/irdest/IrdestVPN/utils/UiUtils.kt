package org.irdest.irdestVPN.utils

import org.irdest.irdestVPN.models.ConnectionState
import org.irdest.irdestVPN.R

fun getColorOfState(state: ConnectionState): Int {
    return when (state) {
        ConnectionState.CONNECTING -> R.color.state_connecting
        ConnectionState.CONNECTED -> R.color.state_connected
        ConnectionState.DISCONNECTED -> R.color.state_disconnected
        ConnectionState.ERROR -> R.color.state_error
        else -> R.color.purple_200
    }
}

fun getBaseMessageOfState(state: ConnectionState): Int {
    return when (state) {
        ConnectionState.CONNECTING -> R.string.connecting
        ConnectionState.CONNECTED -> R.string.connected
        ConnectionState.DISCONNECTED -> R.string.disconnected
        ConnectionState.ERROR -> R.string.Error
        else -> R.string.app_name
    }
}
